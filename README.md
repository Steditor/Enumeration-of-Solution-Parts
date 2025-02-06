# Experiments for Algorithms that Enumerate Solution Parts

This repository contains the code used to run the experiments for \[1, 2\] to investigate algorithms which enumerate solution parts with little preprocessing and delay (instead of computing a full solution and only then returning the whole output).

- \[1\] S. Neubert and K. Casel, “Incremental Ordering for Scheduling Problems,” presented at the 34th International Conference on Automated Planning and Scheduling 2024. Available: <https://openreview.net/forum?id=yXzvVxkYqn>
- \[2\] K. Casel and S. Neubert, “Emit As You Go: Enumerating Edges of a Spanning Tree” (to be presented at the 24th International Conference on Autonomous Agents and Multiagent Systems 2025).

## Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install).
- Either compile the binary with `cargo build --release` and run the built executable or run immediately with `cargo run --release -- <EXPERIMENT_SET> <COMMAND> [OPTIONS]`.
- Show help with `cargo run --release -- --help`.

## Implemented Experiment Sets

```txt
<EXPERIMENT_SET>
        The experiment set to run/aggregate.

        Remember to put the name of the experiment set in quotes, e.g. "F2||C_max"

        [possible values: F2||C_max, P||C_max, 1|prec|C_max, 1|r_j|C_max, MST, SSSD|U|OSM, SSSD|W|OSM, SSSD|U|Artificial, SSSD|W|Artificial, APSD|U|OSM, APSD|W|OSM, APSD|U|Artificial, APSD|W|Artificial, LazyArray]
```

## Running Experiments

```txt
Run the given experiment set

Usage: cargo run --release -- <EXPERIMENT_SET> run [OPTIONS]

Options:
  -m, --max-size <MAX_SIZE>
          Maximum size for instances. Used to limit experiments to small instances during test runs
  -c, --cache-instances [<CACHE_INSTANCES>]
          Write and read generated input instances to/from cache files [default: false] [possible values: true, false]
  -s, --collect-statistics [<COLLECT_STATISTICS>]
          Collect statistics about the input instances [default: true] [possible values: true, false]
  -a, --run-algorithms [<RUN_ALGORITHMS>]
          Run the algorithms [default: true] [possible values: true, false]
  -h, --help
          Print help
```

## Aggregating Data

```txt
Aggregate runtime data of the given experiment set

Usage: cargo run --release -- <EXPERIMENT_SET> aggregate [OPTIONS]

Options:
  -o, --offline
          Use offline aggregation which needs more memory but also computes median and quartiles
  -r, --reference <REFERENCE>
          Extract output quality from algorithm with this name as reference quality for approximation ratios
  -h, --help
          Print help
```

## Download Datasets

```txt
Download datasets.

Usage: cargo run --release --bin datasets -- <DATASET>

Arguments:
  <DATASET>
          The dataset set to download

          Possible values:
          - OpenStreetMap: OpenStreetMap data, downloaded from the links provided in /data/datasets/osm/download-links.csv

Options:
  -h, --help
          Print help
```
