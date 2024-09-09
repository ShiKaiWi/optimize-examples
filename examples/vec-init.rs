use std::time::Instant;

use optimize_examples::ModeParser;

const MODE_MACRO: &str = "macro";
const MODE_RESIZE: &str = "resize";
const MODE_SET_LEN: &str = "setlen";

fn init_with_macro(size: usize, value: u8, count: usize) -> Vec<Vec<u8>> {
    let mut vecs = Vec::with_capacity(count);
    for _ in 0..count {
        vecs.push(vec![value; size]);
    }
    vecs
}

fn init_with_resize(size: usize, value: u8, count: usize) -> Vec<Vec<u8>> {
    let mut vecs = Vec::with_capacity(count);
    for _ in 0..count {
        let mut v = Vec::with_capacity(size);
        v.resize(size, value);
        vecs.push(v);
    }
    vecs
}

fn init_with_set_len(size: usize, value: u8, count: usize) -> Vec<Vec<u8>> {
    let mut vecs = Vec::with_capacity(count);
    for _ in 0..count {
        let mut v = Vec::with_capacity(size);
        unsafe {
            std::ptr::write_bytes(v.as_mut_ptr(), value, size);
            v.set_len(count);
        }
        vecs.push(v);
    }
    vecs
}

fn main() -> Result<(), String> {
    let mode = match ModeParser::parse()? {
        None => MODE_MACRO.to_string(),
        Some(v) => v,
    };
    let size = 10000;
    let count = 100;
    let value = 0;
    let start = Instant::now();

    let vecs = match mode.as_str() {
        MODE_MACRO => init_with_macro(size, value, count),
        MODE_RESIZE => init_with_resize(size, value, count),
        MODE_SET_LEN => init_with_set_len(size, value, count),
        _ => panic!("invalid mode:{mode}"),
    };

    let elapsed = start.elapsed();
    println!(
        "finish with mode:{mode}, cost:{elapsed:?}, num_vectors:{}",
        vecs.len()
    );

    Ok(())
}
