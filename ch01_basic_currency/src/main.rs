use std::collections::VecDeque;
use std::sync::Condvar;
use std::thread;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;


fn main() {
    let f1 = thread::spawn(f);
    let f2 = thread::spawn(f);
    f1.join().unwrap();
    f2.join().unwrap();

    let numbers = vec![1, 2, 3];
    thread::spawn(move || {
        for n in numbers {
            println!("{n}")
        }
    }).join().unwrap();


    let numbers = Vec::from_iter(0..=1000);
    let t = thread::spawn(move || {
        let len = numbers.len();
        let sum = numbers.into_iter().sum::<usize>();
        sum / len
    });
    let average = t.join().unwrap();
    println!("average: {average}");


    let numbers = vec![1,2,3];
    thread::scope(|s|{
        s.spawn(|| {
            println!("length: {}", numbers.len());
        });
        s.spawn(|| {
            for n in &numbers {
                println!("{n}");
            }
        });
    });


    static X: [i32; 3] = [1,2,3];
    thread::spawn(move || dbg!(&X)).join().unwrap();
    thread::spawn(move || dbg!(&X)).join().unwrap();

    let x: &'static [i32; 3]  = Box::leak(Box::new([1,2,3]));
    thread::spawn(move || dbg!(x)).join().unwrap();
    thread::spawn(move || dbg!(x)).join().unwrap();

    let a = Rc::new([1,2,3]);
    let b = a.clone();

    assert_eq!(a.as_ptr(), b.as_ptr());
    println!("{:?} ---- {:?}", a.as_ptr(), b.as_ptr());

    let a = Arc::new([1, 2, 3]);
    let b = a.clone();

    thread::spawn({
        let a = a.clone();
        move || {
            dbg!(a);
        }
    });
    thread::spawn(move || dbg!(a)).join().unwrap();
    thread::spawn(move || dbg!(b)).join().unwrap();


    let n = Mutex::new(0);
    thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| {
                let mut guard = n.lock().unwrap();
                for _ in 0..100 {
                    *guard += 1;
                }
                println!("Thread: {:?}, guard: {}", thread::current().id(), *guard);
                drop(guard);
                thread::sleep(Duration::from_secs(1));
            });
        }
    });

    let queue = Mutex::new(VecDeque::new());
    let not_empty = Condvar::new();

    thread::scope(|s| {
        // Consuming thread
        s.spawn(|| loop {
            let mut q = queue.lock().unwrap();
            let item = loop {
                if let Some(item) = q.pop_front() {
                    break item;
                }else{
                    q = not_empty.wait(q).unwrap();
                }
            };
            drop(q);
            dbg!(item);
        });

        // Producing thread
        for i in 0..=5 {
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_secs(1));
        }
    });


}


fn f(){
    println!("Hello from another thread!");

    let id = thread::current().id();
    println!("This is my thread id: {id:?}");
}
