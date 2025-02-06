use clap::{Parser, ValueEnum};
use exp_lib::data_sets::{osm, DataSet};

#[derive(Parser, Debug)]
#[command(about = "Download datasets.")]
#[command(next_line_help = true)]
struct Args {
    /// The dataset set to download.
    #[arg(value_enum)]
    dataset: Set,
}

#[derive(Clone, ValueEnum, Debug)]
#[allow(clippy::enum_variant_names)]
enum Set {
    /// OpenStreetMap data, downloaded from the links provided in /data/datasets/osm/download-links.csv
    #[clap(name = "OpenStreetMap", alias = "osm")]
    OpenStreetMap,
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let cli = Args::parse();

    let set: DataSet = match cli.dataset {
        Set::OpenStreetMap => osm::DATASET,
    };

    (set.download)().await
}
