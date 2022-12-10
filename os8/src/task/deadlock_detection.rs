use alloc::vec;
use alloc::vec::Vec;

use u32 as SizeType;

#[derive(Clone)]
pub struct DeadlockDetector {
    available: Vec<SizeType>,
    allocation: Vec<Vec<SizeType>>,
    need: Vec<Vec<SizeType>>,
    work: Vec<SizeType>,
    finish: Vec<bool>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            available: vec![],
            allocation: vec![vec![]],
            need: vec![vec![]],
            work: vec![],
            finish: vec![false],
        }
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self {
            available: Default::default(),
            allocation: Default::default(),
            need: Default::default(),
            work: Default::default(),
            finish: Default::default(),
        }
    }
}
