use std::{collections::HashMap, time::Instant};

use optimize_examples::ModeParser;

const MODE_VECTOR: &str = "vector";
const MODE_HASHMAP: &str = "hashmap";

fn init_vector(num: usize) -> Vec<(usize, usize)> {
    let mut v = Vec::with_capacity(num);
    for i in 0..num {
        v.push((i, i * 2));
    }

    v
}

fn init_hashmap(num: usize) -> HashMap<usize, usize> {
    let mut v = HashMap::with_capacity(num);
    for i in 0..num {
        v.insert(i, i * 2);
    }

    v
}

fn do_loop_over_vector(vectors: Vec<Vec<(usize, usize)>>) -> usize {
    let mut value_sum = 0;
    for vector in vectors {
        for (_, value) in vector {
            value_sum += value;
        }
    }

    value_sum
}

fn do_loop_over_hashmap(maps: Vec<HashMap<usize, usize>>) -> usize {
    let mut value_sum = 0;
    for map in maps {
        for (_, value) in map {
            value_sum += value;
        }
    }

    value_sum
}

fn main() -> Result<(), String> {
    let mode = match ModeParser::parse()? {
        None => MODE_VECTOR.to_string(),
        Some(v) => v,
    };

    let num_tasks = 1000;
    let num_elems = 10000;
    let (instant, value_sum) = match mode.as_str() {
        MODE_VECTOR => {
            let mut vectors = Vec::with_capacity(num_tasks);
            for _ in 0..num_tasks {
                let vector = init_vector(num_elems);
                vectors.push(vector);
            }
            let instant = Instant::now();
            (instant, do_loop_over_vector(vectors))
        }
        MODE_HASHMAP => {
            let mut maps = Vec::with_capacity(num_tasks);
            for _ in 0..num_tasks {
                let map = init_hashmap(num_elems);
                maps.push(map);
            }
            let instant = Instant::now();
            (instant, do_loop_over_hashmap(maps))
        }
        _ => panic!("invalid mode"),
    };

    println!("{mode} cost:{:?}, counts:{value_sum}", instant.elapsed());
    Ok(())
}
