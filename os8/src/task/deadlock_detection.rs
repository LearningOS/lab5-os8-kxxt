use alloc::vec;
use alloc::vec::Vec;

use u32 as SizeType;

#[derive(Clone)]
pub struct DeadlockDetector {
    pub available: Vec<SizeType>,
    pub allocation: Vec<Vec<SizeType>>,
    pub need: Vec<Vec<SizeType>>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            available: vec![],
            allocation: vec![vec![]],
            need: vec![vec![]],
        }
    }

    pub fn try_lock(&mut self, tid: usize, rid: usize) {}

    pub fn resize_update_thread_cnt(&mut self, len: usize) {
        let res_len = self.available.len();
        self.allocation.resize(len, vec![0; res_len]);
        self.need.resize(len, vec![0; res_len]);
    }

    pub fn resize_update_res_cnt(&mut self, len: usize) {
        self.available.resize(len, 0);
        for (allocat, nee) in self.allocation.iter_mut().zip(self.need.iter_mut()) {
            allocat.resize(len, 0);
            nee.resize(len, 0);
        }
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self {
            available: Default::default(),
            allocation: Default::default(),
            need: Default::default(),
        }
    }
}
