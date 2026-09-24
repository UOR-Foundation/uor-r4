//! Compiled relative-frame selection; implementation follows the behavioral tests.
#![forbid(unsafe_code)]
use super::RelativeActionModel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRelativePath;

impl CompiledRelativePath {
    pub fn compile(_model: &RelativeActionModel, _path: &[usize]) -> Result<Self, String> {
        Err("compiled relative path not implemented".into())
    }

    pub fn act(&self, _key: [i32; 4]) -> Result<[i32; 4], String> {
        Err("compiled relative path not implemented".into())
    }

    pub fn select(&self, _query: [i32; 4], _keys: &[[i32; 4]]) -> Result<usize, String> {
        Err("compiled relative path not implemented".into())
    }
}
