use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Release, Acquire};
use std::thread;

// #[derive(Debug)]
pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Safety: The very existence of this Guard
        // guarantees we've exclusively locked the lock.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}

// #[derive(Debug)]
pub struct SpinLock<T>{
    locked: AtomicBool,
    value: UnsafeCell<T>,
}


// Sync: SpinLock   is safe to share references across threads
// Send: T          is safe to send to another thread
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self { 
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),

        }
    }

    pub fn lock<'a>(&'a self) -> Guard<T> {
        while self.locked.swap(true, Acquire){
            std::hint::spin_loop();
        }
        Guard { lock: self }
    }

    /// Safety: The &mut T from lock() must be gone!
    /// (And no cheating by keeping reference to fields of that T around!)
    pub fn unlock(&self){
        self.locked.store(false, Release);
    }
}

fn main() {
    let x = SpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| x.lock().push(1));
        s.spawn(|| {
            let mut g = x.lock();
            g.push(2);
            // drop(g); // compiler will complain... dropping Guard too early
            g.push(2);
            drop(g);
        });
        s.spawn(|| {
            let mut g = x.lock();
            g.push(3);
            g.push(3);
        });
    });

    let g =  x.lock();
    println!("{:?}", *g);
}
