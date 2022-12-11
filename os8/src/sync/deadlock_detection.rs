use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use u32 as SizeType;

use crate::sync::UPSafeCell;
use crate::task::TaskControlBlock;

#[derive(Clone)]
pub struct DeadlockDetector {
    pub inner: UPSafeCell<DeadlockDetectorInner>,
}

#[derive(Clone)]
pub struct DeadlockDetectorInner {
    pub available: Vec<SizeType>,
    pub allocation: Vec<Vec<SizeType>>,
    pub need: Vec<Vec<SizeType>>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(DeadlockDetectorInner {
                    available: vec![],
                    allocation: vec![vec![]],
                    need: vec![vec![]],
                })
            },
        }
    }

    pub fn try_lock(&mut self, tid: usize, rid: usize) {}
}

impl DeadlockDetectorInner {
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

    pub fn request(
        &mut self,
        tasks: &[Option<Arc<TaskControlBlock>>],
        tidx: usize,
        rid: usize,
    ) -> bool {
        self.need[tidx][rid] += 1;
        let mut work = self.available.clone();
        // Set finished tasks to true.
        let mut finish: Vec<_> = tasks.iter().map(|t| t.is_none()).collect();
        loop {
            let Some(idx) = (0..tasks.len())
                .find(|&i| finish[i] == false && self.need[i][rid] <= work[rid])
            else {
                break;
            };
            work[rid] += self.allocation[idx][rid];
            finish[idx] = true;
        }
        if finish.iter().any(|&x| x == false) {
            // Rollback changes
            self.need[tidx][rid] -= 1;
            false
        } else {
            true
        }
    }

    pub fn allocate(&mut self, tidx: usize, rid: usize) {
        self.available[rid] -= 1;
        self.need[tidx][rid] -= 1;
        self.allocation[tidx][rid] += 1;
    }

    pub fn free(&mut self, tidx: usize, rid: usize) {
        self.available[rid] += 1;
        self.allocation[tidx][rid] -= 1;
    }
}
