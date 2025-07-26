use crate::parser::FileMeta;
use std::fmt::Debug;

pub trait InsightTypeConfig: Sized + Clone + Debug {
    type F: FileMeta;
}
