use crate::c_mutex::CMutex;
use crate::rwlock::{RwLock, RwLockOp};
use std::cell::UnsafeCell;

pub struct RawFairRwLock {
    r_lock: UnsafeCell<CMutex>,
    w_lock: UnsafeCell<CMutex>,
    reader_n: UnsafeCell<u32>,
}

impl RwLockOp for RawFairRwLock {
    fn new() -> RawFairRwLock {
        unsafe {
            RawFairRwLock {
                r_lock: UnsafeCell::new(CMutex::new()),
                w_lock: UnsafeCell::new(CMutex::new()),
                reader_n: UnsafeCell::new(0),
            }
        }
    }
    unsafe fn lock_reader(&self) {
        let r_lock = &mut *self.r_lock.get();
        r_lock.lock();
        let reader_n = &mut *self.reader_n.get();
        *reader_n += 1;
        if *reader_n == 1 {
            (&mut *self.w_lock.get()).lock();
        }
        r_lock.unlock();
    }
    unsafe fn unlock_reader(&self) {
        let r_lock = &mut *self.r_lock.get();
        r_lock.lock();
        let reader_n = &mut *self.reader_n.get();
        *reader_n -= 1;
        if *reader_n == 0 {
            (&mut *self.w_lock.get()).unlock();
        }
        r_lock.unlock();
    }
    unsafe fn lock_writer(&self) {
        (&mut *self.w_lock.get()).lock();
    }
    unsafe fn unlock_writer(&self) {
        (&mut *self.w_lock.get()).unlock();
    }
}

pub type FairRwLock<T> = RwLock<RawFairRwLock, T>;
