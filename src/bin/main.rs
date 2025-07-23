use anyhow::Result;
use clap::Parser;
use code_insight::cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = cli::Args::parse();
    cli::run(args).await
}