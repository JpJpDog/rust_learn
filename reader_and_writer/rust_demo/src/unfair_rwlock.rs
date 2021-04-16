use crate::c_mutex::CMutex;
use crate::rwlock::{RwLock, RwLockOp};
use std::cell::UnsafeCell;

struct LockContent {
    r_lock: CMutex,
    w_lock: CMutex,
    reader_n: u32,
}

pub struct RawUnfairRwLock(UnsafeCell<LockContent>);

impl RwLockOp for RawUnfairRwLock {
    unsafe fn new() -> RawUnfairRwLock {
        RawUnfairRwLock(UnsafeCell::new(LockContent {
            r_lock: CMutex::new(),
            w_lock: CMutex::new(),
            reader_n: 0,
        }))
    }
    unsafe fn lock_reader(&self) {
        let content = &mut *self.0.get();
        content.r_lock.lock();
        content.reader_n += 1;
        if content.reader_n == 1 {
            content.w_lock.lock()
        }
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
        (&mut *self.0.get()).w_lock.lock();
    }
    unsafe fn unlock_writer(&self) {
        (&mut *self.0.get()).w_lock.unlock();
    }
}

pub type UnfairRwLock<T> = RwLock<RawUnfairRwLock, T>;
