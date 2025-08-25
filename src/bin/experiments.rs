use clap::{Parser, Subcommand, ValueEnum};
use exp_lib::experiments::sets::{
    apsd, apsd_artificial, f2_cmax, lazy_array, mst, p_cmax, prec_cmax, rj_cmax, sorting, sssd,
    sssd_artificial, AggregationOptions, ExperimentOptions, ExperimentSet,
};
use rand::SeedableRng;
use rand_pcg::Pcg64;

#[derive(Parser, Debug)]
#[command(about = "Run experiments or aggregate results for enumeration algorithms.")]
#[command(next_line_help = true)]
struct Args {
    /// The experiment set to run/aggregate.
    ///
    /// Remember to put the name of the experiment set in quotes, e.g. "F2||C_max"
    #[arg(value_enum)]
    experiment_set: Set,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the given experiment set.
    Run {
        #[arg(short = 'm', long)]
        /// Maximum size for instances.
        /// Used to limit experiments to small instances during test runs.
        max_size: Option<u32>,

        /// Write and read generated input instances to/from cache files.
        #[arg(short = 'c', long, default_value_t = false, default_missing_value = "true", num_args = 0..=1)]
        cache_instances: bool,

        /// Collect statistics about the input instances.
        #[arg(short = 's', long, default_value_t = true, default_missing_value = "true", num_args = 0..=1)]
        collect_statistics: bool,

        /// Run the algorithms.
        #[arg(short = 'a', long, default_value_t = true, default_missing_value = "true", num_args = 0..=1)]
        run_algorithms: bool,
    },
    /// Aggregate runtime data of the given experiment set.
    Aggregate {
        /// Use offline aggregation which needs more memory but also computes median and quartiles.
        #[arg(short, long, default_value_t = false)]
        offline: bool,

        /// Extract output quality from algorithm with this name as reference quality for approximation ratios.
        #[arg(short, long)]
        reference: Option<String>,
    },
}

#[derive(Clone, ValueEnum, Debug)]
#[allow(clippy::enum_variant_names)]
enum Set {
    #[clap(name = "F2||C_max", alias = "f2_cmax")]
    F2Cmax,
    #[clap(name = "P||C_max", alias = "p_cmax")]
    PCmax,
    #[clap(name = "1|prec|C_max", alias = "prec_cmax")]
    PrecCmax,
    #[clap(name = "1|r_j|C_max", alias = "rj_cmax")]
    RjCmax,
    #[clap(name = "MST", alias = "mst")]
    Mst,
    #[clap(name = "SSSD|U|OSM", alias = "sssd_u_osm")]
    SingleSourceShortestDistanceUnweightedOSM,
    #[clap(name = "SSSD|W|OSM", alias = "sssd_w_osm")]
    SingleSourceShortestDistanceWeightedOSM,
    #[clap(name = "SSSD|U|Artificial", alias = "sssd_u_a")]
    SingleSourceShortestDistanceUnweightedArtificial,
    #[clap(name = "SSSD|W|Artificial", alias = "sssd_w_a")]
    SingleSourceShortestDistanceWeightedArtificial,
    #[clap(name = "APSD|U|OSM", alias = "apsd_u_osm")]
    AllPairsShortestDistanceUnweightedOSM,
    #[clap(name = "APSD|W|OSM", alias = "apsd_w_osm")]
    AllPairsShortestDistanceWeightedOSM,
    #[clap(name = "APSD|U|Artificial", alias = "apsd_u_a")]
    AllPairsShortestDistanceUnweightedArtificial,
    #[clap(name = "APSD|W|Artificial", alias = "apsd_w_a")]
    AllPairsShortestDistanceWeightedArtificial,
    #[clap(name = "LazyArray", alias = "lazy_array")]
    LazyArray,
    #[clap(name = "Sorting", alias = "sort")]
    Sorting,
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let cli = Args::parse();

    let set: Box<ExperimentSet> = match cli.experiment_set {
        Set::F2Cmax => Box::new(f2_cmax::experiment_set()),
        Set::PCmax => Box::new(p_cmax::experiment_set()),
        Set::PrecCmax => Box::new(prec_cmax::experiment_set()),
        Set::RjCmax => Box::new(rj_cmax::experiment_set()),
        Set::Mst => Box::new(mst::experiment_set()),
        Set::SingleSourceShortestDistanceUnweightedOSM => {
            Box::new(sssd::unweighted_experiment_set())
        }
        Set::SingleSourceShortestDistanceWeightedOSM => Box::new(sssd::weighted_experiment_set()),
        Set::SingleSourceShortestDistanceUnweightedArtificial => {
            Box::new(sssd_artificial::unweighted_experiment_set())
        }
        Set::SingleSourceShortestDistanceWeightedArtificial => {
            Box::new(sssd_artificial::weighted_experiment_set())
        }
        Set::AllPairsShortestDistanceUnweightedOSM => Box::new(apsd::unweighted_experiment_set()),
        Set::AllPairsShortestDistanceWeightedOSM => Box::new(apsd::weighted_experiment_set()),
        Set::AllPairsShortestDistanceUnweightedArtificial => {
            Box::new(apsd_artificial::unweighted_experiment_set())
        }
        Set::AllPairsShortestDistanceWeightedArtificial => {
            Box::new(apsd_artificial::weighted_experiment_set())
        }
        Set::LazyArray => Box::new(lazy_array::experiment_set()),
        Set::Sorting => Box::new(sorting::experiment_set()),
    };

    match cli.command {
        Commands::Aggregate { offline, reference } => {
            (set.aggregate)(&AggregationOptions { offline, reference });
        }
        Commands::Run {
            max_size,
            cache_instances,
            collect_statistics,
            run_algorithms,
        } => (set.run)(&mut ExperimentOptions {
            max_size,
            cache_instances,
            seed_generator: Box::new(Pcg64::seed_from_u64(42)),
            collect_statistics,
            run_algorithms,
        }),
    }
}
