use std::sync::{Arc, Mutex, RwLock};
use std::thread;

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
    use super::{run_mutex, run_rwlock};

    #[test]
    fn test_run_mutex() {
        run_mutex();
    }

    #[test]
    fn test_run_rwlock(){
        run_rwlock();
    }
}
