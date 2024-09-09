use std::time::Instant;

fn add_vector(v0: &[i64], v1: &[i64]) -> Vec<i64> {
    v0.iter().zip(v1.iter()).map(|(x, y)| (*x + *y)).collect()
}

fn chunked_add_vector(v0: &[i64], v1: &[i64]) -> Vec<i64> {
    const CHUNK_SIZE: usize = 1024;

    let mut output = vec![0; v0.len()];
    let chunks0 = v0.chunks_exact(CHUNK_SIZE);
    let chunks1 = v1.chunks_exact(CHUNK_SIZE);
    let output_chunks = output.as_mut_slice().chunks_exact(CHUNK_SIZE);

    for (output_chunk, (ch0, ch1)) in output_chunks.zip(chunks0.zip(chunks1)) {
        let a0: [i64; CHUNK_SIZE] = ch0.try_into().unwrap();
        let a1: [i64; CHUNK_SIZE] = ch1.try_into().unwrap();
        let mut output_chunk: [i64; CHUNK_SIZE] = output_chunk.try_into().unwrap();

        for (idx, (x, y)) in a0.iter().zip(a1.iter()).enumerate() {
            output_chunk[idx] = *x + *y;
        }
    }

    let reminder_len = v0.len() % CHUNK_SIZE;
    let reminder_range = (v0.len() - reminder_len)..v0.len();
    for idx in reminder_range {
        output[idx] = v0[idx] + v1[idx];
    }

    output
}

fn main() {
    let sz = 64 * 1024;

    let mut v0 = Vec::with_capacity(sz);
    let mut v1 = Vec::with_capacity(sz);

    for i in 0..sz {
        v0.push(i as i64);
        v1.push(i as i64);
    }

    {
        let start = Instant::now();
        let v2 = add_vector(&v0, &v1);
        let elapsed = start.elapsed();
        let sum: i64 = v2.into_iter().sum();
        println!("sum:{sum}, elapsed:{elapsed:?}");
    }

    {
        let start = Instant::now();
        let v2 = chunked_add_vector(&v0, &v1);
        let elapsed = start.elapsed();
        let sum: i64 = v2.into_iter().sum();
        println!("chunked sum:{sum}, elapsed:{elapsed:?}");
    }
}
