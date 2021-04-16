use crate::c_mutex::CMutex;
use libc::{pthread_cond_broadcast, pthread_cond_t, pthread_cond_wait, PTHREAD_COND_INITIALIZER};
pub struct CCond(pthread_cond_t);

impl CCond {
    pub unsafe fn new() -> CCond {
        CCond(PTHREAD_COND_INITIALIZER)
    }

    pub unsafe fn wait(&mut self, mutex: &mut CMutex) {
        pthread_cond_wait(&mut self.0, &mut mutex.m);
    }

    pub unsafe fn broadcast(&mut self) {
        pthread_cond_broadcast(&mut self.0);
    }
}
