use clap::{Parser, ValueEnum};
use exp_lib::experiments::sets::{f2_cmax, prec_cmax, rj_cmax, ExperimentOptions, ExperimentSet};

#[derive(Parser, Debug)]
#[command(about = "Run experiments or aggregate results for enumeration algorithms.")]
#[command(next_line_help = true)]
struct Args {
    /// The experiment set to run/aggregate.
    ///
    /// Remember to put the name of the experiment set in quotes, e.g. "F2||C_max"
    #[arg(value_enum)]
    experiment_set: Set,

    /// Aggregate results from a previous run of the experiment set instead of running the experiments.
    #[arg(short, long, default_value_t = false)]
    aggregate: bool,

    #[arg(short, long)]
    /// Maximum size for instances.
    /// Used to limit experiments to small instances during test runs.
    max_size: Option<u32>,

    /// Write and read generated input instances to/from cache files.
    #[arg(short, long, default_value_t = false)]
    cache_instances: bool,
}

#[derive(Clone, ValueEnum, Debug)]
#[allow(clippy::enum_variant_names)]
enum Set {
    #[clap(name = "F2||C_max", alias = "f2_cmax")]
    F2Cmax,
    #[clap(name = "1|prec|C_max", alias = "prec_cmax")]
    PrecCmax,
    #[clap(name = "1|r_j|C_max", alias = "rj_cmax")]
    RjCmax,
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let cli = Args::parse();

    let set: Box<ExperimentSet> = match cli.experiment_set {
        Set::F2Cmax => Box::new(f2_cmax::experiment_set()),
        Set::PrecCmax => Box::new(prec_cmax::experiment_set()),
        Set::RjCmax => Box::new(rj_cmax::experiment_set()),
    };

    if cli.aggregate {
        (set.aggregate)();
    } else {
        (set.run)(ExperimentOptions {
            max_size: cli.max_size,
            cache_instances: cli.cache_instances,
        })
    }
}
