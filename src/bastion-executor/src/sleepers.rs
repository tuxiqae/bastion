//!
//! Where workers went to parking while no workload is in their worker queue.
//!
//! If a workload received pool will wake them up.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

/// The place where worker threads go to sleep.
///
/// Similar to how thread parking works, if a notification comes up while no threads are sleeping,
/// the next thread that attempts to go to sleep will pick up the notification immediately.
#[derive(Debug)]
#[allow(clippy::mutex_atomic)]
pub struct Sleepers {
    /// How many threads are currently a sleep.
    sleep: Mutex<usize>,

    /// A condvar for notifying sleeping threads.
    wake: Condvar,

    /// Set to `true` if a notification came up while nobody was sleeping.
    notified: AtomicBool,
}

#[allow(clippy::mutex_atomic)]
impl Default for Sleepers {
    /// Creates a new `Sleepers`.
    fn default() -> Self {
        Self {
            sleep: Mutex::new(0),
            wake: Condvar::new(),
            notified: AtomicBool::new(false),
        }
    }
}

#[allow(clippy::mutex_atomic)]
impl Sleepers {
    /// Creates a new `Sleepers`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts the current thread to sleep.
    pub fn wait(&self) {
        let mut sleep = self.sleep.lock().unwrap();

        if !self.notified.swap(false, Ordering::SeqCst) {
            *sleep += 1;
            std::mem::drop(self.wake.wait(sleep).unwrap());
        }
    }

    /// Notifies one thread.
    pub fn notify_one(&self) {
        if !self.notified.load(Ordering::SeqCst) {
            let mut sleep = self.sleep.lock().unwrap();

            if *sleep > 0 {
                *sleep -= 1;
                self.wake.notify_one();
            } else {
                self.notified.store(true, Ordering::SeqCst);
            }
        }
    }
}
