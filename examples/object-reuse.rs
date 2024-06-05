use optimize_examples::ModeParser;

use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Instant,
};

use crossbeam::queue::SegQueue;
use threadpool::ThreadPool;

const SMALL_SIZE: usize = 64 + 1;
const MIDDLE_SIZE: usize = 512 + 1;
const LARGE_SIZE: usize = 4 * 1024 + 1;
const MODE_ALLOC: &str = "alloc";
const MODE_REUSE: &str = "reuse";

struct Pool {
    small_objs: SegQueue<Vec<u8>>,
    middle_objs: SegQueue<Vec<u8>>,
    large_objs: SegQueue<Vec<u8>>,
}

impl Pool {
    fn new() -> Self {
        Self {
            small_objs: SegQueue::new(),
            middle_objs: SegQueue::new(),
            large_objs: SegQueue::new(),
        }
    }

    fn choose_pool(&self, size: usize) -> &SegQueue<Vec<u8>> {
        if size <= SMALL_SIZE {
            &self.small_objs
        } else if size <= MIDDLE_SIZE {
            &self.middle_objs
        } else {
            &self.large_objs
        }
    }

    fn get(&self, size: usize) -> (Vec<u8>, bool) {
        let pool = self.choose_pool(size);
        match pool.pop() {
            Some(mut v) => {
                v.clear();
                (v, false)
            }
            None => (Vec::with_capacity(size), true),
        }
    }

    fn put(&self, size: usize, obj: Vec<u8>) {
        let pool = self.choose_pool(size);
        pool.push(obj);
    }
}

fn alloc_and_init(n: usize) -> usize {
    let mut all_vectors = Vec::with_capacity(n * 3);
    for _ in 0..n {
        all_vectors.push(do_alloc_and_init(SMALL_SIZE));
        all_vectors.push(do_alloc_and_init(MIDDLE_SIZE));
        all_vectors.push(do_alloc_and_init(LARGE_SIZE));
    }

    all_vectors.into_iter().map(|v| v.len()).sum()
}

fn do_alloc_and_init(size: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(size);
    do_init(&mut v, size);
    v
}

fn do_init(vector: &mut Vec<u8>, n: usize) {
    for i in 0..n {
        vector.push((i % 256) as u8);
    }
}

fn do_reuse_and_init(pool: &Pool, size: usize) -> (Vec<u8>, bool) {
    let (mut v, new) = pool.get(size);
    do_init(&mut v, size);
    (v, new)
}

fn reuse_and_init(pool: &Pool, n: usize) -> usize {
    let mut all_vectors = Vec::with_capacity(n * 3);
    for _ in 0..n {
        all_vectors.push(do_reuse_and_init(pool, SMALL_SIZE));
        all_vectors.push(do_reuse_and_init(pool, MIDDLE_SIZE));
        all_vectors.push(do_reuse_and_init(pool, LARGE_SIZE));
    }

    let mut allocated_bytes = 0;
    for (v, new) in all_vectors {
        if new {
            allocated_bytes += v.len();
        }
        pool.put(v.len(), v);
    }

    allocated_bytes
}

fn run_allocate_and_init(num_threads: usize, num_tasks: usize, num_alloc: usize) {
    let thread_pool = ThreadPool::new(num_threads);
    let allocated_bytes = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();
    for _ in 0..num_tasks {
        let allocated_bytes = allocated_bytes.clone();
        thread_pool.execute(move || {
            let n = alloc_and_init(num_alloc);
            allocated_bytes.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        });
    }

    thread_pool.join();
    println!(
        "allocate_and_init cost:{:?}, allocated bytes:{}",
        start.elapsed(),
        allocated_bytes.load(std::sync::atomic::Ordering::Relaxed)
    );
}

fn run_reuse_and_init(num_threads: usize, num_tasks: usize, num_alloc: usize) {
    let thread_pool = ThreadPool::new(num_threads);
    let allocated_bytes = Arc::new(AtomicUsize::new(0));
    let mut pools = Vec::with_capacity(num_threads);
    for _ in 0..num_threads {
        pools.push(Arc::new(Pool::new()));
    }

    let start = Instant::now();
    for idx in 0..num_tasks {
        let allocated_bytes = allocated_bytes.clone();
        let pool = pools[idx % num_threads].clone();
        thread_pool.execute(move || {
            let n = reuse_and_init(pool.as_ref(), num_alloc);
            allocated_bytes.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        });
    }

    thread_pool.join();
    println!(
        "reuse_and_init cost:{:?}, allocated bytes:{}",
        start.elapsed(),
        allocated_bytes.load(std::sync::atomic::Ordering::Relaxed)
    );
}

/// Two modes are defined:
/// - alloc
/// - reuse
fn main() -> Result<(), String> {
    let mode = match ModeParser::parse()? {
        None => MODE_ALLOC.to_string(),
        Some(v) => v,
    };

    let num_threads = 4;
    let num_tasks = 4096;
    let num_alloc = 1000;
    match mode.as_str() {
        MODE_REUSE => run_reuse_and_init(num_threads, num_tasks, num_alloc),
        _ => run_allocate_and_init(num_threads, num_tasks, num_alloc),
    }

    Ok(())
}
