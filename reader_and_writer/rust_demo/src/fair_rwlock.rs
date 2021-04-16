use crate::c_cond::CCond;
use crate::c_mutex::CMutex;
use crate::rwlock::{RwLock, RwLockOp};
use std::cell::UnsafeCell;

struct LockContent {
    r_lock: CMutex,
    w_lock: CMutex,
    w_wait_cond: CCond,
    reader_n: u32,
    last_writer_id: u32,
    next_writer_id: u32,
}

pub struct RawFairRwLock(UnsafeCell<LockContent>);

impl RwLockOp for RawFairRwLock {
    unsafe fn new() -> RawFairRwLock {
        RawFairRwLock(UnsafeCell::new(LockContent {
            r_lock: CMutex::new(),
            w_lock: CMutex::new(),
            w_wait_cond: CCond::new(),
            reader_n: 0,
            last_writer_id: 0,
            next_writer_id: 1,
        }))
    }

    unsafe fn lock_reader(&self) {
        let content = &mut *self.0.get();
        content.r_lock.lock();
        if content.reader_n == 0 {
            content.w_lock.lock();
        } else {
            let wait_writer_id = content.next_writer_id - 1;
            while content.last_writer_id < wait_writer_id {
                content.w_wait_cond.wait(&mut content.r_lock);
            }
        }
        content.reader_n += 1;
        content.r_lock.unlock();
    }

    unsafe fn unlock_reader(&self) {
        let content = &mut *self.0.get();
        content.r_lock.lock();
        content.reader_n -= 1;
        if content.reader_n == 0 {
            content.w_lock.unlock();
        }
        content.r_lock.unlock();
    }

    unsafe fn lock_writer(&self) {
        let content = &mut *self.0.get();
        content.r_lock.lock();
        let writer_id = content.next_writer_id;
        content.next_writer_id += 1;
        content.r_lock.unlock();
        content.w_lock.lock();
        content.last_writer_id = writer_id;
    }

    unsafe fn unlock_writer(&self) {
        let content = &mut *self.0.get();
        content.w_wait_cond.broadcast();
        content.w_lock.unlock();
    }
}

pub type FairRwLock<T> = RwLock<RawFairRwLock, T>;
