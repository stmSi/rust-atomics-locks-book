use std::sync::atomic::{AtomicI32, AtomicU64, AtomicBool, AtomicPtr, fence};
use std::sync::atomic::Ordering::{Release, Acquire, Relaxed, SeqCst};
use std::thread;
use std::time::Duration;

fn main(){
    static mut DATA: [u64; 10] = [0; 10];

    const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
    static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

    for i in 0..10{
        thread::spawn(move || {
            let data: u64 = i + 1; // some calculations
            thread::sleep(Duration::from_millis(500));
            unsafe { DATA[i as usize] = data };
            READY[i as usize].store(true, Release);
        });
    }
    thread::sleep(Duration::from_millis(500));
    let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Relaxed));

    if ready.contains(&true) {
        fence(Acquire);
        for i in 0..10 {
            if ready[i] {
                println!("data{i} = {}", unsafe {DATA[i]});
            }
        }
    }
}

fn main_06(){
    static A: AtomicBool = AtomicBool::new(false);
    static B: AtomicBool = AtomicBool::new(false);

    static mut S: String = String::new();
    let a = thread::spawn(|| {
        A.store(true, SeqCst);
        if !B.load(SeqCst){
            unsafe { S.push('!')};
        }
    });

    let b = thread::spawn(|| {
        B.store(true, SeqCst);
        if !A.load(SeqCst) {
            unsafe {S.push('!')};
        }
    });

    a.join().unwrap();
    b.join().unwrap();
}

fn main_05(){
    static X: AtomicI32 = AtomicI32::new(0);
    #[derive(Debug)]
    struct Data{}
    fn get_data() -> &'static Data {
        static PTR: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());
        println!("{:?} is trying to acquire {:?}", thread::current().id(), PTR);
        let mut p = PTR.load(Acquire);
        thread::sleep(Duration::from_millis(100)); 

        if p.is_null(){
            p = Box::into_raw(Box::new(Data{})); // generate data
            if let Err(e) = PTR.compare_exchange(
                std::ptr::null_mut(), p, Release, Acquire) {
                println!("{:?} is failed to exchanged {:?}", thread::current().id(), PTR);
                // Safety: p comes from Box::into_raw right above,
                // and wasn't shared with any other thread.
                drop(unsafe {
                    Box::from_raw(p)
                });
                p = e;
            }else{
                println!("{:?} compare and exchange data {:?}", thread::current().id(), PTR);
            }

        }        // Safety: p is not null and points to a properly initialized vaue.
        //
        //
        println!("{:?} received {:?}", thread::current().id(), p);
        unsafe {&*p}
    } // end of get_data()
    
    fn want_to_use_data(){
        let d = get_data();

        println!("{:?} bye! {:?}", thread::current().id(), d);
    }
    thread::scope(|s| {
        s.spawn(|| want_to_use_data());
        s.spawn(|| want_to_use_data());
        s.spawn(|| want_to_use_data());
        s.spawn(|| want_to_use_data());
        s.spawn(|| want_to_use_data());
    });
}

fn main_04(){
    static mut DATA: String = String::new();
    static LOCKED: AtomicBool = AtomicBool::new(false);

    {
        fn f(){
            if LOCKED.compare_exchange(false, true, Acquire, Relaxed).is_ok() {
                // Safety: We hold the exclusive lock, so nothing else is accessing the DATA.
                unsafe { DATA.push('!')};
                println!("pushing... {:?}", thread::current().id());
                LOCKED.store(false, Release);
            }
        }

        thread::scope(|s|{
            for _ in 0..10000{
                s.spawn(f);
            }
        });
    }
}

fn main_03(){
    static mut DATA: u64 = 0;
    static READY: AtomicBool = AtomicBool::new(false);

    {// main thread

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

    
    {// main thread

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
    static X: AtomicI32 = AtomicI32::new(0);
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
