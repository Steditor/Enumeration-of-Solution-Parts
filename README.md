# Experiments for Algorithms that Enumerate Solution Parts

This repository contains the code used to run the experiments for \[1\] to investigate algorithms which enumerate solution parts with little preprocessing and delay (instead of computing a full solution and only then returning the whole output).

- \[1\] S. Neubert and K. Casel, “Incremental Ordering for Scheduling Problems,” presented at the 34th International Conference on Automated Planning and Scheduling 2024. Available: <https://openreview.net/forum?id=yXzvVxkYqn>

## Prerequisites

- install [Rust](https://www.rust-lang.org/tools/install)
- either compile the binary with `cargo build --release` and run the built executable or run immediately with `cargo run --release -- [OPTIONS] <EXPERIMENT_SET>`

## Running Experiments

```txt
Run experiments or aggregate results for enumeration algorithms.

Usage: experiments.exe [OPTIONS] <EXPERIMENT_SET>

Arguments:
  <EXPERIMENT_SET>
          The experiment set to run/aggregate.

          Remember to put the name of the experiment set in quotes, e.g. "F2||C_max"

          [possible values: F2||C_max, 1|prec|C_max, 1|r_j|C_max]

Options:
  -a, --aggregate
          Aggregate results from a previous run of the experiment set instead of running the experiments

  -m, --max-size <MAX_SIZE>
          Maximum size for instances. Used to limit experiments to small instances during test runs

  -h, --help
          Print help (see a summary with '-h')
```
