use std::cell::UnsafeCell;
use std::marker::{Send, Sync};
use std::ops::{Deref, DerefMut};

pub trait RwLockOp {
    unsafe fn new() -> Self;
    unsafe fn lock_reader(&self);
    unsafe fn unlock_reader(&self);
    unsafe fn lock_writer(&self);
    unsafe fn unlock_writer(&self);
}

pub struct RwLock<R: RwLockOp, T> {
    raw: R,
    data: UnsafeCell<T>,
}

pub struct RwLockReaderGuard<'a, R: RwLockOp, T> {
    rwlock: &'a RwLock<R, T>,
}

pub struct RwLockWriterGuard<'a, R: RwLockOp, T> {
    rwlock: &'a RwLock<R, T>,
}

unsafe impl<R: RwLockOp, T> Send for RwLock<R, T> {}

unsafe impl<R: RwLockOp, T> Sync for RwLock<R, T> {}

impl<'a, R: RwLockOp, T> Drop for RwLockReaderGuard<'a, R, T> {
    fn drop(&mut self) {
        unsafe {
            self.rwlock.raw.unlock_reader();
        }
    }
}

impl<'a, R: RwLockOp, T> Drop for RwLockWriterGuard<'a, R, T> {
    fn drop(&mut self) {
        unsafe {
            self.rwlock.raw.unlock_writer();
        }
    }
}

impl<'a, R: RwLockOp, T> Deref for RwLockReaderGuard<'a, R, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.data.get() }
    }
}

impl<'a, R: RwLockOp, T> Deref for RwLockWriterGuard<'a, R, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.data.get() }
    }
}

impl<'a, R: RwLockOp, T> DerefMut for RwLockWriterGuard<'a, R, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.rwlock.data.get() }
    }
}

impl<R: RwLockOp, T> RwLock<R, T> {
    pub fn new(data: T) -> RwLock<R, T> {
        unsafe {
            RwLock {
                raw: R::new(),
                data: UnsafeCell::new(data),
            }
        }
    }

    pub fn read(&self) -> RwLockReaderGuard<R, T> {
        unsafe {
            self.raw.lock_reader();
        }
        RwLockReaderGuard { rwlock: self }
    }

    pub fn write(&self) -> RwLockWriterGuard<R, T> {
        unsafe {
            self.raw.lock_writer();
        }
        RwLockWriterGuard { rwlock: self }
    }
}
