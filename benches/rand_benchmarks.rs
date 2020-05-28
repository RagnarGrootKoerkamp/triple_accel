use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rand::prelude::*;
use triple_accel::*;
use triple_accel::levenshtein::*;
use triple_accel::hamming::*;

fn bench_rand_hamming(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(1234);
    let (a_str, b_str) = black_box(rand_hamming_pair(1000, 30, &mut rng));

    let res = hamming_naive(&a_str, &b_str);
    assert!(res == hamming_words_64(&a_str, &b_str));
    assert!(res == hamming_words_128(&a_str, &b_str));
    assert!(res == hamming_simd_movemask(&a_str, &b_str));
    assert!(res == hamming_simd_parallel(&a_str, &b_str));

    let mut group = c.benchmark_group("bench_rand_hamming");

    group.bench_function("hamming_naive", |b| b.iter(|| hamming_naive(&a_str, &b_str)));
    group.bench_function("hamming_words_64", |b| b.iter(|| hamming_words_64(&a_str, &b_str)));
    group.bench_function("hamming_words_128", |b| b.iter(|| hamming_words_128(&a_str, &b_str)));
    group.bench_function("hamming_simd_movemask", |b| b.iter(|| hamming_simd_movemask(&a_str, &b_str)));
    group.bench_function("hamming_simd_parallel", |b| b.iter(|| hamming_simd_parallel(&a_str, &b_str)));

    group.finish();
}

fn bench_rand_hamming_search(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(1234);
    let k = black_box(16);
    let (needle, haystack) = black_box(rand_hamming_needle_haystack(32, 1000, 50, k, &mut rng));

    let res = hamming_search_naive_with_opts(&needle, &haystack, k, SearchType::All);
    assert!(res == hamming_search_simd_with_opts(&needle, &haystack, k, SearchType::All));

    let mut group = c.benchmark_group("bench_rand_hamming_search");

    group.bench_function("hamming_search_naive_k", |b| b.iter(|| hamming_search_naive_with_opts(&needle, &haystack, k, SearchType::All)));
    group.bench_function("hamming_search_simd_k", |b| b.iter(|| hamming_search_simd_with_opts(&needle, &haystack, k, SearchType::All)));

    group.finish();
}

fn bench_rand_levenshtein(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(1234);
    let (a_str, b_str) = black_box(rand_levenshtein_pair(1000, 100, &mut rng));

    let res = levenshtein_naive(&a_str, &b_str);
    assert!(res == levenshtein_exp(&a_str, &b_str));
    assert!(res == levenshtein(&a_str, &b_str));

    let mut group = c.benchmark_group("bench_rand_levenshtein");

    group.bench_function("levenshtein_naive", |b| b.iter(|| levenshtein_naive(&a_str, &b_str)));
    group.bench_function("levenshtein_exp", |b| b.iter(|| levenshtein_exp(&a_str, &b_str)));
    group.bench_function("levenshtein", |b| b.iter(|| levenshtein(&a_str, &b_str)));

    group.finish();
}

fn bench_rand_levenshtein_k(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(1234);
    let k = black_box(30);
    let trace_on = false;
    let (a_str, b_str) = black_box(rand_levenshtein_pair(1000, k, &mut rng));

    let res = levenshtein_naive_with_opts(&a_str, &b_str, trace_on, LEVENSHTEIN_COSTS);
    assert!(res == levenshtein_naive_k_with_opts(&a_str, &b_str, k, trace_on, LEVENSHTEIN_COSTS).unwrap());
    assert!(res == levenshtein_simd_k_with_opts(&a_str, &b_str, k, trace_on, LEVENSHTEIN_COSTS).unwrap());

    let mut group = c.benchmark_group("bench_rand_levenshtein_k");

    group.bench_function("levenshtein_naive", |b| b.iter(|| levenshtein_naive_with_opts(&a_str, &b_str, trace_on, LEVENSHTEIN_COSTS)));
    group.bench_function("levenshtein_naive_k", |b| b.iter(|| levenshtein_naive_k_with_opts(&a_str, &b_str, k, trace_on, LEVENSHTEIN_COSTS)));
    group.bench_function("levenshtein_simd_k", |b| b.iter(|| levenshtein_simd_k_with_opts(&a_str, &b_str, k, trace_on, LEVENSHTEIN_COSTS)));

    group.finish();
}

fn bench_rand_levenshtein_search(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(1234);
    let k = black_box(16);
    let (needle, haystack) = black_box(rand_levenshtein_needle_haystack(32, 1000, 50, k, &mut rng));

    let res = levenshtein_search_naive_with_opts(&needle, &haystack, k, SearchType::All, LEVENSHTEIN_COSTS, false);
    assert!(res == levenshtein_search_simd_with_opts(&needle, &haystack, k, SearchType::All, LEVENSHTEIN_COSTS, false));

    let mut group = c.benchmark_group("bench_rand_levenshtein_search");

    group.bench_function("levenshtein_search_naive_k", |b| b.iter(|| levenshtein_search_naive_with_opts(&needle, &haystack, k, SearchType::All, LEVENSHTEIN_COSTS, false)));
    group.bench_function("levenshtein_search_simd_k", |b| b.iter(|| levenshtein_search_simd_with_opts(&needle, &haystack, k, SearchType::All, LEVENSHTEIN_COSTS, false)));

    group.finish();
}

criterion_group!(bench_rand, bench_rand_hamming, bench_rand_hamming_search, bench_rand_levenshtein, bench_rand_levenshtein_k, bench_rand_levenshtein_search);
criterion_main!(bench_rand);

fn rand_hamming_needle_haystack<R: Rng>(needle_len: usize, haystack_len: usize, num_match: usize, k: u32, rng: &mut R) -> (Vec<u8>, Vec<u8>) {
    let mut idx: Vec<usize> = (0usize..haystack_len).collect();
    idx.shuffle(rng);
    let mut insert = vec![false; haystack_len];

    for i in 0..num_match {
        insert[idx[i]] = true;
    }

    let bytes: Vec<u8> = (33u8..127u8).collect();
    let needle = rand_alloc_str(needle_len, rng);
    let mut haystack: Vec<u8> = vec![];

    for i in 0..haystack_len {
        if insert[i] {
            let s = rand_hamming_mutate(&needle, k, rng);
            haystack.extend(&s[..needle_len]);
        }else{
            haystack.push(*bytes.choose(rng).unwrap());
        }
    }

    let mut haystack_final = alloc_str(haystack.len());
    fill_str(&mut haystack_final, &haystack);

    (needle, haystack_final)
}

fn rand_hamming_pair<R: Rng>(length: usize, k: u32, rng: &mut R) -> (Vec<u8>, Vec<u8>) {
    let a = rand_alloc_str(length, rng);
    let b = rand_hamming_mutate(&a, k, rng);

    (a, b)
}

fn rand_hamming_mutate<R: Rng>(a: &[u8], k: u32, rng: &mut R) -> Vec<u8> {
    let mut b = alloc_str(a.len());
    fill_str(&mut b, a);
    let curr_k: usize = rng.gen_range((k / 2) as usize, k as usize + 1);
    let mut idx: Vec<usize> = (0usize..a.len()).collect();
    idx.shuffle(rng);

    for i in 0..curr_k {
        b[idx[i]] = 32u8;
    }

    b
}

fn rand_levenshtein_needle_haystack<R: Rng>(needle_len: usize, haystack_len: usize, num_match: usize, k: u32, rng: &mut R) -> (Vec<u8>, Vec<u8>) {
    let mut idx: Vec<usize> = (0usize..haystack_len).collect();
    idx.shuffle(rng);
    let mut insert = vec![false; haystack_len];

    for i in 0..num_match {
        insert[idx[i]] = true;
    }

    let bytes: Vec<u8> = (33u8..127u8).collect();
    let needle = rand_str(needle_len, rng);
    let mut haystack: Vec<u8> = vec![];

    for i in 0..haystack_len {
        if insert[i] {
            let s = rand_levenshtein_mutate(&needle, k, rng);
            haystack.extend(&s);
        }else{
            haystack.push(*bytes.choose(rng).unwrap());
        }
    }

    (needle, haystack)
}

fn rand_levenshtein_pair<R: Rng>(length: usize, k: u32, rng: &mut R) -> (Vec<u8>, Vec<u8>) {
    let a = rand_str(length, rng);
    let b = rand_levenshtein_mutate(&a, k, rng);

    (a, b)
}

fn rand_levenshtein_mutate<R: Rng>(a: &[u8], k: u32, rng: &mut R) -> Vec<u8> {
    let mut edits = vec![0u8; a.len()];
    let curr_k: usize = rng.gen_range((k / 2) as usize, k as usize + 1);
    let mut idx: Vec<usize> = (0usize..a.len()).collect();
    idx.shuffle(rng);

    for i in 0..curr_k {
        edits[idx[i]] = rng.gen_range(1u8, 4u8);
    }

    let bytes: Vec<u8> = (33u8..127u8).collect();
    let mut b = vec![];

    for i in 0..a.len() {
        match edits[i] {
            0u8 => { // same
                b.push(a[i]);
            },
            1u8 => { // diff
                b.push(32u8);
            },
            2u8 => { // insert
                b.push(*bytes.choose(rng).unwrap());
                b.push(a[i]);
            },
            3u8 => (), // delete
            _ => panic!("This should not have been reached!")
        }
    }

    b
}

fn rand_str<R: Rng>(length: usize, rng: &mut R) -> Vec<u8> {
    let bytes: Vec<u8> = (33u8..127u8).collect();
    let mut res = vec![0u8; length];

    for i in 0..length {
        res[i] = *bytes.choose(rng).unwrap();
    }

    res
}

fn rand_alloc_str<R: Rng>(length: usize, rng: &mut R) -> Vec<u8> {
    let bytes: Vec<u8> = (33u8..127u8).collect();
    let mut res = alloc_str(length);

    for i in 0..length {
        res[i] = *bytes.choose(rng).unwrap();
    }

    res
}

