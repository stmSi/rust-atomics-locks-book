use std::sync::atomic::{AtomicI32, AtomicU64, AtomicBool};
use std::sync::atomic::Ordering::{Release, Acquire, Relaxed};
use std::thread;
use std::time::Duration;
static X: AtomicI32 = AtomicI32::new(0);

fn main(){
    static mut DATA: u64 = 0;
    static READY: AtomicBool = AtomicBool::new(false);

    // main thread
    {
       thread::spawn(|| {
            // Safety: Nothing else is accessing DATA,
            // because we haven't set the READY flag yet.
            unsafe{ DATA = 123 };
            READY.store(true, Release); // Everything from before this store...
        });
        while !READY.load(Acquire) { // .. is visible after this loads `true`.
            thread::sleep(Duration::from_millis(100));
            println!("waiting...");
        }
        // Safety: Nothing is mutating DATA, because READY is set.
        println!("{}", unsafe { DATA });
    }
}

fn main_02(){
    static DATA: AtomicU64 = AtomicU64::new(0);
    static READY: AtomicBool = AtomicBool::new(false);

    // main thread
    {
       thread::spawn(|| {
            DATA.store(123, Relaxed);
            READY.store(true, Release); // Everything from before this store...
        });
        while !READY.load(Acquire) { // .. is visible after this loads `true`.
            thread::sleep(Duration::from_millis(100));
            println!("waiting...");
        }
        println!("{}", DATA.load(Relaxed));
    }
}

fn main_01() {
    X.store(1, Relaxed);
    let t = thread::spawn(f);
    thread::sleep(Duration::from_micros(5)); // wait 5 micro seconds
    X.store(2, Relaxed);
    t.join().unwrap();
    X.store(3, Relaxed);

    fn f(){
        let x = X.load(Relaxed);
        assert!(x == 1 || x == 2);
        println!("x: {x}");
    }
}
