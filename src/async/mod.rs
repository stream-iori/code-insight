use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio::sync::{Semaphore, mpsc};
use rayon::prelude::*;

use crate::{
    parser::{FileParser, JavaParser},
    indexer::IndexManager,
    types::JavaFile,
};

#[derive(Clone)]
pub struct AsyncProcessor {
    max_concurrent_files: usize,
    max_concurrent_parsers: usize,
    semaphore: Arc<Semaphore>,
}

impl AsyncProcessor {
    pub fn new(max_concurrent_files: usize, max_concurrent_parsers: usize) -> Self {
        Self {
            max_concurrent_files,
            max_concurrent_parsers,
            semaphore: Arc::new(Semaphore::new(max_concurrent_files)),
        }
    }

    pub async fn process_project_async(
        &self,
        project_root: &Path,
        index_manager: Arc<IndexManager>,
    ) -> Result<ProcessingStats> {
        let file_parser = FileParser::new()?;
        let java_files = file_parser.find_source_files(project_root)?
            .into_iter()
            .filter(|p| p.extension().map_or(false, |e| e == "java"))
            .collect::<Vec<_>>();

        let stats = Arc::new(std::sync::Mutex::new(ProcessingStats::new()));
        let (tx, mut rx): (mpsc::Sender<Result<JavaFile>>, mpsc::Receiver<Result<JavaFile>>) = mpsc::channel(1000);

        // Spawn async tasks for file processing
        let mut join_set = JoinSet::new();
        let files_chunk_size = (java_files.len() / self.max_concurrent_files).max(1);

        for chunk in java_files.chunks(files_chunk_size) {
            let chunk = chunk.to_vec();
            let tx = tx.clone();
            let stats = stats.clone();
            let semaphore = self.semaphore.clone();

            join_set.spawn(async move {
                for file_path in chunk {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    match Self::process_single_file(&file_path).await {
                        Ok(java_file) => {
                            let _ = tx.send(Ok(java_file)).await;
                            stats.lock().unwrap().increment_processed();
                        }
                        Err(e) => {
                            eprintln!("Error processing {}: {}", file_path.display(), e);
                            stats.lock().unwrap().increment_errors();
                        }
                    }
                }
            });
        }

        // Spawn indexer task
        let indexer_handle = tokio::spawn({
            let index_manager = index_manager.clone();
            async move {
                let mut processed = 0;
                while let Some(result) = rx.recv().await {
                    match result {
                        Ok(java_file) => {
                            if let Err(e) = index_manager.index_java_file(&java_file).await {
                                eprintln!("Error indexing file: {}", e);
                            } else {
                                processed += 1;
                                if processed % 100 == 0 {
                                    println!("📊 Indexed {} files...", processed);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                processed
            }
        });

        // Wait for all processing to complete
        while let Some(_) = join_set.join_next().await {}
        drop(tx); // Close the channel

        let total_indexed = indexer_handle.await?;
        
        let final_stats = stats.lock().unwrap().clone();
        println!("✅ Async processing completed. Indexed {} files", total_indexed);
        
        Ok(final_stats)
    }

    async fn process_single_file(file_path: &PathBuf) -> Result<JavaFile> {
        let mut java_parser = JavaParser::new()?;
        java_parser.parse_file(file_path)
    }

    pub async fn process_files_parallel(
        &self,
        files: Vec<PathBuf>,
    ) -> Result<Vec<Result<JavaFile>>> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_parsers));
        let mut join_set = JoinSet::new();

        for file_path in files {
            let semaphore = semaphore.clone();
            
            join_set.spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                Self::process_single_file(&file_path).await
            });
        }

        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            results.push(result.unwrap());
        }

        Ok(results)
    }

    pub fn process_files_rayon(
        &self,
        files: Vec<PathBuf>,
    ) -> Result<Vec<Result<JavaFile>>> {
        let results: Vec<Result<JavaFile>> = files
            .into_par_iter()
            .map(|file_path| {
                let mut java_parser = JavaParser::new()?;
                java_parser.parse_file(&file_path)
            })
            .collect();

        Ok(results)
    }

    pub async fn process_with_backpressure(
        &self,
        project_root: &Path,
        index_manager: Arc<IndexManager>,
        batch_size: usize,
    ) -> Result<ProcessingStats> {
        let file_parser = FileParser::new()?;
        let java_files = file_parser.find_source_files(project_root)?
            .into_iter()
            .filter(|p| p.extension().map_or(false, |e| e == "java"))
            .collect::<Vec<_>>();

        let stats = Arc::new(std::sync::Mutex::new(ProcessingStats::new()));
        let (tx, rx): (mpsc::Sender<Result<JavaFile>>, mpsc::Receiver<Result<JavaFile>>) = mpsc::channel(batch_size);

        let producer = tokio::spawn({
            let java_files = java_files.clone();
            let self_clone = self.clone();
            async move {
                for chunk in java_files.chunks(batch_size) {
                    let chunk = chunk.to_vec();
                    let results = self_clone.process_files_rayon(chunk)?;
                    
                    for result in results {
                        tx.send(result).await.unwrap();
                    }
                }
                Result::<(), anyhow::Error>::Ok(())
            }
        });

        // Consumer task
        let consumer = tokio::spawn({
            let stats = stats.clone();
            let index_manager = index_manager.clone();
            async move {
                let mut processed = 0;
                let mut rx = rx;
                
                while let Some(result) = rx.recv().await {
                    match result {
                        Ok(java_file) => {
                            if let Err(e) = index_manager.index_java_file(&java_file).await {
                                eprintln!("Error indexing: {}", e);
                                stats.lock().unwrap().increment_errors();
                            } else {
                                processed += 1;
                                stats.lock().unwrap().increment_processed();
                                
                                if processed % 100 == 0 {
                                    println!("🔄 Processed {} files with backpressure...", processed);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Processing error: {}", e);
                            stats.lock().unwrap().increment_errors();
                        }
                    }
                }
                processed
            }
        });

        producer.await??;
        let total_processed = consumer.await?;
        
        let final_stats = stats.lock().unwrap().clone();
        println!("🎯 Backpressure processing completed. Processed {} files", total_processed);
        
        Ok(final_stats)
    }

    pub async fn monitor_progress(
        &self,
        project_root: &Path,
    ) -> Result<ProgressMonitor> {
        let file_parser = FileParser::new()?;
        let total_files = file_parser.find_source_files(project_root)?
            .into_iter()
            .filter(|p| p.extension().map_or(false, |e| e == "java"))
            .count();

        Ok(ProgressMonitor::new(total_files))
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub total_files: usize,
    pub processed_files: usize,
    pub error_files: usize,
    pub start_time: std::time::Instant,
}

impl ProcessingStats {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            processed_files: 0,
            error_files: 0,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn increment_processed(&mut self) {
        self.processed_files += 1;
    }

    pub fn increment_errors(&mut self) {
        self.error_files += 1;
    }

    pub fn progress(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            self.processed_files as f64 / self.total_files as f64
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn processing_rate(&self) -> f64 {
        let elapsed_secs = self.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 {
            self.processed_files as f64 / elapsed_secs
        } else {
            0.0
        }
    }
}

impl Default for ProcessingStats {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProgressMonitor {
    total_files: usize,
    processed_files: Arc<std::sync::atomic::AtomicUsize>,
    error_files: Arc<std::sync::atomic::AtomicUsize>,
    start_time: std::time::Instant,
}

impl ProgressMonitor {
    pub fn new(total_files: usize) -> Self {
        Self {
            total_files,
            processed_files: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            error_files: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn increment_processed(&self) {
        self.processed_files.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.error_files.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn progress(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            self.processed_files.load(std::sync::atomic::Ordering::Relaxed) as f64 / self.total_files as f64
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn print_progress(&self) {
        let processed = self.processed_files.load(std::sync::atomic::Ordering::Relaxed);
        let errors = self.error_files.load(std::sync::atomic::Ordering::Relaxed);
        let progress = self.progress();
        let elapsed = self.elapsed();
        let rate = if elapsed.as_secs() > 0 {
            processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        println!("Progress: {:.1}% ({}/{} files, {} errors, {:.2} files/sec)",
            progress * 100.0,
            processed,
            self.total_files,
            errors,
            rate
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_async_processor() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let index_path = dir.path().join("index");
        
        let index_manager = IndexManager::new(&index_path).unwrap();
        let processor = AsyncProcessor::new(4, 2);
        
        // Create test Java file
        let test_file = project_root.join("Test.java");
        std::fs::write(&test_file, r#"
            public class Test {
                private int value;
                public void setValue(int v) { this.value = v; }
            }
        "#).unwrap();
        
        let stats = processor.process_project_async(project_root, Arc::new(index_manager)).await.unwrap();
        
        assert!(stats.processed_files >= 1);
        assert_eq!(stats.error_files, 0);
    }

    #[tokio::test]
    async fn test_progress_monitor() {
        let monitor = ProgressMonitor::new(100);
        
        assert_eq!(monitor.progress(), 0.0);
        
        monitor.increment_processed();
        assert_eq!(monitor.progress(), 0.01);
        
        monitor.increment_processed();
        assert_eq!(monitor.progress(), 0.02);
    }

    #[test]
    fn test_processing_stats() {
        let mut stats = ProcessingStats::new();
        stats.total_files = 100;
        
        assert_eq!(stats.progress(), 0.0);
        
        stats.increment_processed();
        assert_eq!(stats.progress(), 0.01);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(stats.processing_rate() > 0.0);
    }
}