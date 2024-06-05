use crossbeam_utils::CachePadded;
use optimize_examples::ModeParser;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use threadpool::ThreadPool;

const MODE_PADDED: &str = "padded";
const MODE_RAW: &str = "raw";
const MODE_PARTITIONED: &str = "partitioned";

fn do_with_buffer(buffer: &mut Vec<u8>, data_size: usize) {
    for i in 0..data_size {
        buffer.push(i as u8);
    }
}

struct PaddedPartitionedBuffer {
    buffers: Vec<CachePadded<Mutex<Vec<u8>>>>,
}

fn insert_into_padded_partitioned_buffer(
    num_threads: usize,
    num_tasks: usize,
    data_size: usize,
) -> usize {
    let thread_pool = ThreadPool::new(num_threads);
    let rate = 4;
    let buffer = {
        let num_partitions = num_threads * rate;
        let mut buffers = Vec::with_capacity(num_partitions);
        let partition_buffer_cap = (data_size * num_tasks) / num_partitions;
        for _ in 0..num_partitions {
            let buffer = Vec::with_capacity(partition_buffer_cap);
            buffers.push(CachePadded::new(Mutex::new(buffer)));
        }

        Arc::new(PaddedPartitionedBuffer { buffers })
    };

    for i in 0..num_tasks {
        let buffer = buffer.clone();
        thread_pool.execute(move || {
            let idx = i % buffer.buffers.len();
            let mut buffer = buffer.buffers[idx].lock().unwrap();
            do_with_buffer(&mut buffer, data_size);
        });
    }

    thread_pool.join();

    buffer.buffers.iter().map(|v| v.lock().unwrap().len()).sum()
}

struct PartitionedBuffer {
    buffers: Vec<Mutex<Vec<u8>>>,
}

fn insert_into_partitioned_buffer(num_threads: usize, num_tasks: usize, data_size: usize) -> usize {
    let thread_pool = ThreadPool::new(num_threads);
    let rate = 4;
    let buffer = {
        let num_partitions = num_threads * rate;
        let mut buffers = Vec::with_capacity(num_partitions);
        let partition_buffer_cap = (data_size * num_tasks) / num_partitions;
        for _ in 0..num_partitions {
            let buffer = Vec::with_capacity(partition_buffer_cap);
            buffers.push(Mutex::new(buffer));
        }

        Arc::new(PartitionedBuffer { buffers })
    };

    for i in 0..num_tasks {
        let buffer = buffer.clone();
        thread_pool.execute(move || {
            let idx = i % buffer.buffers.len();
            let mut buffer = buffer.buffers[idx].lock().unwrap();
            do_with_buffer(&mut buffer, data_size);
        });
    }

    thread_pool.join();
    buffer.buffers.iter().map(|v| v.lock().unwrap().len()).sum()
}

struct Buffer {
    buffer: Mutex<Vec<u8>>,
}

fn insert_into_raw_buffer(num_threads: usize, num_tasks: usize, data_size: usize) -> usize {
    let thread_pool = ThreadPool::new(num_threads);
    let buffer = Arc::new(Buffer {
        buffer: Mutex::new(Vec::with_capacity(data_size * num_tasks)),
    });
    for _ in 0..num_tasks {
        let buffer = buffer.clone();
        thread_pool.execute(move || {
            let mut buffer = buffer.buffer.lock().unwrap();
            do_with_buffer(&mut buffer, data_size);
        });
    }

    thread_pool.join();
    let n = buffer.buffer.lock().unwrap().len();
    n
}

fn main() -> Result<(), String> {
    let mode = match ModeParser::parse()? {
        None => MODE_RAW.to_string(),
        Some(v) => v,
    };

    let instant = Instant::now();
    let num_threads = 4;
    let num_tasks = 1024 * 1024;
    let data_size = 4096;
    let num_inserted_bytes = match mode.as_str() {
        MODE_PADDED => insert_into_padded_partitioned_buffer(num_threads, num_tasks, data_size),
        MODE_PARTITIONED => insert_into_partitioned_buffer(num_threads, num_tasks, data_size),
        _ => insert_into_raw_buffer(num_threads, num_tasks, data_size),
    };

    println!(
        "{mode} cost:{:?}, inserted_bytes:{num_inserted_bytes}",
        instant.elapsed()
    );
    Ok(())
}
