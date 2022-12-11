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

#[derive(Clone, Debug)]
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
        debug!("begin self = {:?}, finish = {:?}", self, finish);
        loop {
            let mut found = false;
            'outer: for i in 0..tasks.len() {
                // for all unfinished tasks
                if finish[i] {
                    // to make the borrow checker happy.
                    // I didn't use .filter(|&x| finish[x] == false)
                    continue;
                }
                // check it!
                for r in 0..self.available.len() {
                    if self.need[i][r] > work[r] {
                        // Not a valid thread/task
                        continue 'outer;
                    }
                }
                for r in 0..self.available.len() {
                    work[r] += self.allocation[i][r];
                }
                finish[i] = true;
                found = true;
            }
            if !found {
                break;
            }
        }
        debug!(
            "end self = {:?}, finish = {:?}, work = {:?}",
            self, finish, work
        );
        if finish.iter().any(|&x| x == false) {
            // Rollback changes
            self.need[tidx][rid] -= 1;
            false
        } else {
            debug!("LGTM");
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
