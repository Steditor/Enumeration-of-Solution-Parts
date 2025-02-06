#![feature(iter_map_windows)]

use std::{
    fs::File,
    io::{BufWriter, Write},
};

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let k = 40_000_u32;
    let n = k * k + k;
    let m = (k * (k - 1) / 2 + k * k) as usize;

    log::info!("k: {k}, n: {n}, m: {m}");

    let edge_file = File::create("tmp.txt").unwrap();
    let mut edge_writer = BufWriter::new(edge_file);

    // build an almost-k-clique between vertices {0, 1, ..., k-1} with one missing edge (k-2, k-1).
    (0..k - 2)
        .flat_map(|u| (u + 1..k).map(move |v| (u, v)))
        .for_each(|(u, v)| {
            //edges.push((u, v, 1));
            writeln!(edge_writer, "{u}, {v}").unwrap();
        });

    // build a path from k-1 to k-2 that runs over all remaining vertices
    (k - 1..n)
        .chain(std::iter::once(k - 2))
        .map_windows(|[u, v]| (*u, *v))
        .for_each(|(u, v)| {
            //edges.push((u, v, 1));
            writeln!(edge_writer, "{u}, {v}").unwrap();
        });
}
