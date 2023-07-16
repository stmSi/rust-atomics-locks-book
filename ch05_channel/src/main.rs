use std::{
    cell::UnsafeCell,
    collections::VecDeque,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::{atomic::AtomicBool, mpsc::channel, Arc, Condvar, Mutex},
    thread::{self, Thread},
    time::Duration,
};

fn main() {
    struct Channel<T> {
        // no longer `pub`
        message: UnsafeCell<MaybeUninit<T>>,
        ready: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    pub struct Sender<'a, T> {
        channel: &'a Channel<T>,
        receiving_thread: Thread,
    }

    pub struct Receiver<'a, T> {
        channel: &'a Channel<T>,

        // *const () doesn't implment Send... so that
        // receiver can't be sent to another thread.
        _no_send: PhantomData<*const ()>,
    }

    impl<T> Channel<T> {
        pub const fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                ready: AtomicBool::new(false),
            }
        }

        pub fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
            *self = Self::new();
            (
                Sender {
                    channel: self,
                    receiving_thread: thread::current(),
                },
                Receiver {
                    channel: self,
                    _no_send: PhantomData,
                },
            )
        }
    }

    impl<T> Sender<'_, T> {
        /// This neveer panics. :)
        pub fn send(self, message: T) {
            unsafe { (*self.channel.message.get()).write(message) };
            self.channel
                .ready
                .store(true, std::sync::atomic::Ordering::Release);
            self.receiving_thread.unpark();
        }
    }

    impl<T> Receiver<'_, T> {
        pub fn is_ready(&self) -> bool {
            self.channel
                .ready
                .load(std::sync::atomic::Ordering::Relaxed)
        }

        pub fn receive(self) -> T {
            while !self
                .channel
                .ready
                .swap(false, std::sync::atomic::Ordering::Acquire)
            {
                thread::park();
            }
            unsafe { (*self.channel.message.get()).assume_init_read() }
        }
    }

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe { self.message.get_mut().assume_init_drop() }
            }
        }
    }

    // Main
    let mut channel = Channel::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        s.spawn(move || {
            thread::sleep(Duration::from_millis(100));
            sender.send("hello world!");
        });
        println!("received: {:?}", receiver.receive());
    });

    main_01()
}

fn main_01() {
    pub struct Channel<T> {
        message: UnsafeCell<MaybeUninit<T>>,
        ready: AtomicBool,
        in_use: AtomicBool,
    }

    unsafe impl<T> Sync for Channel<T> where T: Send {}

    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            if *self.ready.get_mut() {
                unsafe { self.message.get_mut().assume_init_drop() }
            }
        }
    }

    impl<T> Channel<T> {
        pub fn new() -> Self {
            Self {
                message: UnsafeCell::new(MaybeUninit::uninit()),
                ready: AtomicBool::new(false),
                in_use: AtomicBool::new(false),
            }
        }

        /// Panics when trying to send more than one message.
        pub fn send(&self, message: T) {
            if self.in_use.swap(true, std::sync::atomic::Ordering::Relaxed) {
                panic!("can't send more than one message!");
            }
            unsafe { (*self.message.get()).write(message) };
            self.ready.store(true, std::sync::atomic::Ordering::Release);
        }

        pub fn is_ready(&self) -> bool {
            self.ready.load(std::sync::atomic::Ordering::Relaxed)
        }

        /// Panics if no message is available yet.
        /// or if the messaaaage was already consumed.
        /// Tip: Use `is_ready` to check first.
        ///
        /// Safety: Only call this once,
        /// and only after is_ready() returns true!
        pub fn receive(&self) -> T {
            if !self.ready.swap(false, std::sync::atomic::Ordering::Acquire) {
                panic!("no message available!");
            }
            // Safety: We've just checked (and reset) the ready flag.
            unsafe { (*self.message.get()).assume_init_read() }
        }
    }

    // Main
    let channel = Channel::new();
    let t = thread::current();
    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_millis(100));
            channel.send("Hello World from Chaneel!");
            t.unpark();
        });
        while !channel.is_ready() {
            println!("Parking....");
            thread::park();
        }
        println!("received: {:?}", channel.receive());
    });
}
