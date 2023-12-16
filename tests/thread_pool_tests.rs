use std::sync::{Arc, Mutex};

use alex_lib::threads::ThreadPool;

#[test]
fn test_thread_pool() {
    let mut tp = ThreadPool::new(3);
    let res = Arc::new(Mutex::new(Vec::new()));

    for _ in 0..3 {
        let res = res.clone();
        tp.enqueue(move |id| {
            res.lock().unwrap().push((id, id * id));
        });
    }
    tp.graceful_shutdown();
    let res = res.lock().unwrap();
    assert_eq!(res.len(), 3);
    for i in 0..3 as usize {
        let (id, idsqr) = res[i];
        assert_eq!([1, 2, 3].contains(&(id as i64)), true);
        assert_eq!(idsqr, id * id);
    }
}
