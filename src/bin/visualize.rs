#![feature(iter_map_windows)]

use clap::{command, Parser, ValueEnum};
use exp_lib::visualization::mst;

#[derive(Parser, Debug)]
#[command(about = "Visualize enumeration algorithms.")]
#[command(next_line_help = true)]
struct Args {
    /// The algorithm to visualize.
    #[arg(value_enum)]
    visualization: Visualization,
}

#[derive(Clone, ValueEnum, Debug)]
#[allow(clippy::enum_variant_names)]
enum Visualization {
    #[clap(name = "MST", alias = "mst")]
    Mst,
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let cli = Args::parse();

    match cli.visualization {
        Visualization::Mst => mst::visualize().unwrap(),
    }
}
