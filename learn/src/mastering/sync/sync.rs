use std::sync::mpsc::{self, channel};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

fn run_mpsc_sync_channel() {
    let (tx, rx) = mpsc::sync_channel(1);
    let tx_clone = tx.clone();

    let _ = tx.send(0);

    thread::spawn(move || {
        let _ = tx.send(1);
    });

    thread::spawn(move || {
        let _ = tx_clone.send(2);
    });

    println!("Received {} via the channel", rx.recv().unwrap());
    println!("Received {} via the channel", rx.recv().unwrap());
    println!("Received {} via the channel", rx.recv().unwrap());
    println!("Received {:?} via the channel", rx.recv());
}

fn run_mpsc() {
    let (tx, rx) = channel();
    let join_handle = thread::spawn(move || {
        while let Ok(n) = rx.recv() {
            println!("Received {}", n);
        }
    });
    for i in 0..10 {
        tx.send(i).unwrap();
    }
    join_handle.join().unwrap();
}

// 出现了死锁，注释其中一个joinHandler，可以解锁
fn run_mpsc_multi_tx() {
    let (tx, rx) = channel();
    let mut childs = vec![];

    let join_handle = thread::spawn(move || {
        // loop{
        //     match rx.recv() {
        //         Ok(n) => println!("Received {}", n),
        //         Err(e) => {
        //             println!("Received {}", e);
        //             panic!("{}", e)
        //         }
        //     }
        // }
        while let Ok(n) = rx.recv() {
            println!("Received {}", n);
        }
    });
    for i in 0..10 {
        let v = tx.clone();
        let t = thread::spawn(move || {
            v.send(i).unwrap();
        });
        childs.push(t);
    }

    for c in childs {
        c.join().unwrap();
    }
    // join_handle.join().unwrap();
}

fn run_mutex() {
    let vec = Arc::new(Mutex::new(vec![]));
    let mut childs = vec![];
    for i in 0..5 {
        let mut v = vec.clone();
        let t = thread::spawn(move || {
            let mut v = v.lock().unwrap();
            v.push(i);
        });
        childs.push(t);
    }
    for c in childs {
        c.join().unwrap();
    }
    println!("{:?}", vec);
}

// RwLock on some systems such as Linux, suffers from the writer starvation problem.
fn run_rwlock() {
    let m = RwLock::new(5);
    let c = thread::spawn(move || {
        {
            *m.write().unwrap() += 1;
        }
        let updated = *m.read().unwrap();
        updated
    });
    let updated = c.join().unwrap();
    println!("{:?}", updated);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_run_mutex() {
        run_mutex();
    }

    #[test]
    fn test_run_rwlock() {
        run_rwlock();
    }

    #[test]
    fn test_run_mpsc() {
        run_mpsc();
    }

    #[test]
    fn test_run_mpsc_multi_tx() {
        run_mpsc_multi_tx();
    }

    #[test]
    fn test_run_mpsc_sync_channel() {
        run_mpsc_sync_channel();
    }
}
