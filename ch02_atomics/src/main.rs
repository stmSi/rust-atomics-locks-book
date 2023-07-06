use std::sync::atomic::{AtomicUsize, AtomicU64};
use std::time::{Duration, Instant};
use std::{sync::atomic::AtomicBool, thread};
use std::sync::atomic::Ordering::Relaxed;

fn main(){
    fn get_key() -> u64{
        static KEY: AtomicU64 = AtomicU64::new(0);
        let key = KEY.load(Relaxed);
        if key == 0 {
            let new_key = 48295421; // random key
            match  KEY.compare_exchange(0, new_key, Relaxed, Relaxed) {
                Ok(_) => new_key,
                Err(k) => k,
            }
        }else{
            key
        }
    }
}

fn main_04(){
    let num_done = &AtomicUsize::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);
    thread::scope(|s| {
        // Four backdground threads to process all 100 items, 25 each.
        for t in 0..4{
            s.spawn(move || {
                for i in 0..25{
                    let start = Instant::now();
                    // Very long processing
                    thread::sleep(Duration::from_millis(10));

                    let time_taken = start.elapsed().as_micros() as u64;

                    num_done.fetch_add(1, Relaxed);
                    total_time.fetch_add(time_taken, Relaxed);
                    max_time.fetch_max(time_taken, Relaxed);
                }
            });
        }
        
        // The main thread show status updates, every second.
        loop {
            let total_time = Duration::from_micros(total_time.load(Relaxed));
            let max_time = Duration::from_micros(max_time.load(Relaxed));

            let n = num_done.load(Relaxed);
            if n == 100 {break;}
            if n == 0 {
                println!("Working... nothing done yet.");
            }else{
                println!("Working.. {n}/100 done, {:?} average, {:?} peak",
                    total_time / n as u32,
                    max_time
                );
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    println!("Done");
}

fn increment(a: &AtomicU64) {
    let mut current = a.load(Relaxed);
    loop {
        let new = current + 1;
        match a.compare_exchange(current, new, Relaxed, Relaxed){
            Ok(_) => return,
            Err(v) => current = v,
        }
    }
}

fn main_03(){
    fn get_x() -> u64{
        static X: AtomicU64= AtomicU64::new(0);
        let mut x = X.load(Relaxed);
        if x == 0 {
            // Calculate x
            println!("Calculating x...");
            x = 69;
            X.store(x, Relaxed);
        }
        x
    }

    get_x(); // will only print one time.
    get_x();
    get_x();
    get_x();
    get_x();

}

fn main_02(){
    let num_done = AtomicUsize::new(0);

    let main_thread = thread::current();

    thread::scope(|s| {
        // A background thred to process all 100 items.
        s.spawn(|| {
            for i in 0..100{
                // do some operation that take some time.
                thread::sleep(Duration::from_millis(90));
                num_done.store(i + 1, Relaxed);
                main_thread.unpark(); // Wake up the main thread
            }
        });

        // the main thread shows status updates. every seconds.
        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {break;}
            println!("Working.. {n}/100 done");
            thread::park_timeout(Duration::from_secs(1));
        }
    });

    println!("Done!");

}



fn main_01() {
    static STOP: AtomicBool = AtomicBool::new(false);

    // Spawn a thread to do the work.
    let background_thread = thread::spawn(|| {
        while !STOP.load(Relaxed) {
            //
        }
    });

    // Use the main thread to listen for user input.
    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("commands: help, stop"),
            "stop" => break,
            cmd => println!("Unknown command: {cmd:?}"),
        }
    }

    // Inform the background thread it needs to stop.
    STOP.store(true, Relaxed);


    // Wait Until the background thread finishes.
    background_thread.join().unwrap();
}
