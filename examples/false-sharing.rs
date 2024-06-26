use crossbeam_utils::CachePadded;
use optimize_examples::ModeParser;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

const MODE_PADDED: &str = "padded";
const MODE_RAW: &str = "raw";

struct CounterSet {
    c0: AtomicUsize,
    c1: AtomicUsize,
}

struct PaddedCounterSet {
    c0: CachePadded<AtomicUsize>,
    c1: CachePadded<AtomicUsize>,
}

fn count_over_raw(num_tasks: usize, num_counts: usize) -> usize {
    let counter_set = Arc::new(CounterSet {
        c0: AtomicUsize::new(0),
        c1: AtomicUsize::new(0),
    });
    let c0 = counter_set.clone();
    let h0 = std::thread::spawn(move || {
        for _ in 0..num_tasks {
            for _ in 0..num_counts {
                c0.c0.fetch_add(1, Ordering::Relaxed);
            }
        }
    });
    let c1 = counter_set.clone();
    let h1 = std::thread::spawn(move || {
        for _ in 0..num_tasks {
            for _ in 0..num_counts {
                c1.c1.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    h0.join().unwrap();
    h1.join().unwrap();

    counter_set.c0.load(Ordering::Relaxed) + counter_set.c1.load(Ordering::Relaxed)
}

fn count_over_padded(num_tasks: usize, num_counts: usize) -> usize {
    let counter_set = Arc::new(PaddedCounterSet {
        c0: CachePadded::new(AtomicUsize::new(0)),
        c1: CachePadded::new(AtomicUsize::new(0)),
    });

    let c0 = counter_set.clone();
    let h0 = std::thread::spawn(move || {
        for _ in 0..num_tasks {
            for _ in 0..num_counts {
                c0.c0.fetch_add(1, Ordering::Relaxed);
            }
        }
    });
    let c1 = counter_set.clone();
    let h1 = std::thread::spawn(move || {
        for _ in 0..num_tasks {
            for _ in 0..num_counts {
                c1.c1.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    h0.join().unwrap();
    h1.join().unwrap();

    counter_set.c0.load(Ordering::Relaxed) + counter_set.c1.load(Ordering::Relaxed)
}

fn main() -> Result<(), String> {
    let mode = match ModeParser::parse()? {
        None => MODE_RAW.to_string(),
        Some(v) => v,
    };

    let instant = Instant::now();
    let num_tasks = 1024 * 64;
    let num_counts = 4096;
    let counts = match mode.as_str() {
        MODE_PADDED => count_over_padded(num_tasks, num_counts),
        MODE_RAW => count_over_raw(num_tasks, num_counts),
        _ => panic!("invalid mode"),
    };

    println!("{mode} cost:{:?}, counts:{counts}", instant.elapsed());
    Ok(())
}
