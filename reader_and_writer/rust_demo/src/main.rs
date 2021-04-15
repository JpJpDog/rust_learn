mod c_mutex;
mod fair_rwlock;
mod rwlock;

use fair_rwlock::FairRwLock;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::Arc;
use std::thread;
use std::thread::spawn;
use std::time::Duration;

const THREAD_N: usize = 10;
const WRITER_THREAD_N: usize = 3;
const READER_LOOP_N: u32 = 200;
const WRITER_LOOP_N: u32 = 40;
const MAX_SLEEP_TIME: u64 = 100;
const MIN_SLEEP_TIME: u64 = 10;

fn rand_sleep_time() -> u64 {
    let mut rng = rand::thread_rng();
    let rand_int = rng.gen::<u64>();
    rand_int % (MAX_SLEEP_TIME - MIN_SLEEP_TIME) + MIN_SLEEP_TIME
}

fn reader_routine(rwlock: &FairRwLock<i32>) {
    let data = rwlock.read();
    println!("data is {}", *data);
    thread::sleep(Duration::from_millis(rand_sleep_time()))
}

fn writer_routine(rwlock: &FairRwLock<i32>) {
    let mut data = rwlock.write();
    *data += 1;
    println!("data is {} now", *data);
    thread::sleep(Duration::from_millis(rand_sleep_time()))
}

fn routine(is_writer: bool, rwlock: &FairRwLock<i32>) {
    if is_writer {
        for _ in 0..WRITER_LOOP_N {
            writer_routine(rwlock);
        }
    } else {
        for _ in 0..READER_LOOP_N {
            reader_routine(rwlock);
        }
    }
}

fn main() {
    let mut handlers = vec![];
    let mut is_writer = vec![false; THREAD_N];
    for i in 0..WRITER_THREAD_N {
        is_writer[i] = true;
    }
    is_writer.shuffle(&mut thread_rng());
    let rwlock = Arc::new(FairRwLock::new(0));
    for flag in is_writer {
        let rwlock = Arc::clone(&rwlock);
        let handler = spawn(move || routine(flag, rwlock.as_ref()));
        handlers.push(handler);
    }
    for handler in handlers {
        handler.join().unwrap();
    }
}
