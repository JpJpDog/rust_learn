use libc::{pthread_mutex_lock, pthread_mutex_t, pthread_mutex_unlock, PTHREAD_MUTEX_INITIALIZER};

pub struct CMutex {
    pub m: pthread_mutex_t,
}

impl CMutex {
    pub unsafe fn new() -> CMutex {
        CMutex {
            m: PTHREAD_MUTEX_INITIALIZER,
        }
    }

    pub unsafe fn lock(&mut self) {
        pthread_mutex_lock(&mut self.m);
    }

    pub unsafe fn unlock(&mut self) {
        pthread_mutex_unlock(&mut self.m);
    }
}
