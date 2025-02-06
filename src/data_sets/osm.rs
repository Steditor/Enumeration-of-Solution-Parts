use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use num::cast::AsPrimitive;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::{
    data_structures::graphs::{Graph, UndirectedAdjacencyArrayGraph},
    io::{self, download::download_file},
};

use super::{DataSet, DataSetReaderError, GraphReader};

const ROOT: &str = "./data/datasets/osm/";

#[derive(Default)]
pub struct OsmReaderOptions {
    pub max_size: Option<u32>,
    pub required_tags: Option<Vec<(String, String)>>,
    pub merge_ways: bool,
}
impl OsmReaderOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_max_size(mut self, max_size: Option<u32>) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn require_tag(mut self, key: &str, value: &str) -> Self {
        self.required_tags
            .get_or_insert_default()
            .push((key.to_string(), value.to_string()));
        self
    }

    pub fn merge_ways(mut self, merge: bool) -> Self {
        self.merge_ways = merge;
        self
    }
}

pub struct OsmReader {}

impl OsmReader {
    pub fn get_file_paths() -> Box<dyn Iterator<Item = PathBuf>> {
        Box::new(
            WalkDir::new(ROOT)
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|e| e.into_path())
                .filter(|e| {
                    e.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".osm.pbf"))
                }),
        )
    }

    pub fn get_folders() -> Vec<PathBuf> {
        let files = Self::get_file_paths();
        let mut unique_folders = HashSet::new();
        for file in files {
            if let Some(folder) = file.parent() {
                unique_folders.insert(folder.to_path_buf());
            }
        }
        unique_folders.drain().collect()
    }

    pub fn read_from_file(
        path: &Path,
        options: &OsmReaderOptions,
    ) -> Result<(Vec<osm4routing::Node>, Vec<osm4routing::Edge>), DataSetReaderError> {
        let mut reader = osm4routing::Reader::new();
        if let Some(tags) = &options.required_tags {
            for tag in tags {
                reader = reader.require(&tag.0, &tag.1);
            }
        }
        if options.merge_ways {
            reader = reader.merge_ways();
        }
        reader.read(path).map_err(|why| {
            DataSetReaderError::InputError(format!(
                "Could not read osm data from {}: {}",
                path.display(),
                why
            ))
        })
    }

    fn prepare_import(
        path: &Path,
        options: &OsmReaderOptions,
    ) -> Result<(HashMap<i64, u32>, Vec<osm4routing::Edge>), DataSetReaderError> {
        let (osm_nodes, osm_edges) = Self::read_from_file(path, options)?;

        assert!(osm_nodes.len() < u32::MAX as usize);
        let num_vertices = osm_nodes.len() as u32;
        if options.max_size.is_some_and(|max| num_vertices > max) {
            return Err(DataSetReaderError::TooLarge(
                num_vertices,
                options.max_size.expect("Checked in if clause."),
            ));
        }

        let mut node_translation = HashMap::with_capacity(osm_nodes.len());
        for (index, node) in osm_nodes.iter().enumerate() {
            node_translation.insert(node.id.0, index as u32);
        }

        Ok((node_translation, osm_edges))
    }

    fn get_edge_endpoints(
        edge: &osm4routing::Edge,
        node_translation: &HashMap<i64, u32>,
    ) -> Result<(u32, u32), DataSetReaderError> {
        let u =
            node_translation
                .get(&edge.source.0)
                .ok_or(DataSetReaderError::ConsistencyError(format!(
                    "Missing source node {} for edge to {}.",
                    edge.source.0, edge.target.0,
                )))?;
        let v =
            node_translation
                .get(&edge.target.0)
                .ok_or(DataSetReaderError::ConsistencyError(format!(
                    "Missing target node {} for edge from {}.",
                    edge.source.0, edge.target.0,
                )))?;
        Ok((*u, *v))
    }
}

impl GraphReader<UndirectedAdjacencyArrayGraph<u32, ()>, u32, (), OsmReaderOptions> for OsmReader {
    fn read_from(
        path: &Path,
        options: &OsmReaderOptions,
    ) -> Result<UndirectedAdjacencyArrayGraph<u32, ()>, DataSetReaderError> {
        let (node_translation, osm_edges) = Self::prepare_import(path, options)?;
        let num_vertices = node_translation.len() as u32;

        let mut unique_edges = HashSet::with_capacity(osm_edges.len());
        for edge in osm_edges {
            let (u, v) = Self::get_edge_endpoints(&edge, &node_translation)?;
            unique_edges.insert((u.min(v), u.max(v)));
        }

        let edges: Vec<_> = unique_edges.drain().collect();
        Ok(UndirectedAdjacencyArrayGraph::new(num_vertices, &edges))
    }
}

impl GraphReader<UndirectedAdjacencyArrayGraph<u32, u32>, u32, u32, OsmReaderOptions>
    for OsmReader
{
    fn read_from(
        path: &Path,
        options: &OsmReaderOptions,
    ) -> Result<UndirectedAdjacencyArrayGraph<u32, u32>, DataSetReaderError> {
        let (node_translation, osm_edges) = Self::prepare_import(path, options)?;
        let num_vertices = node_translation.len() as u32;

        let mut unique_edges = HashSet::with_capacity(osm_edges.len());
        for edge in osm_edges {
            let (u, v) = Self::get_edge_endpoints(&edge, &node_translation)?;
            // we cast edge length in meters from f64 to u32, which should be enough precision for our applications
            unique_edges.insert((u.min(v), u.max(v), edge.length().as_()));
        }

        let edges: Vec<_> = unique_edges.drain().collect();
        Ok(UndirectedAdjacencyArrayGraph::new_with_edge_data(
            num_vertices,
            &edges,
        ))
    }
}

pub const DATASET: DataSet = DataSet {
    download: || Box::pin(download_osm_links()),
};

#[derive(Default, Deserialize, Debug)]
struct DownloadLink {
    path: String,
    url: String,
}

pub async fn download_osm_links() {
    let download_links_path = Path::new(ROOT).join("download-links.csv");
    let links = match io::csv::read_from_file::<DownloadLink>(&download_links_path) {
        Err(why) => {
            log::error!(
                "Reading the download links from {} failed: {}",
                download_links_path.display(),
                why
            );
            return;
        }
        Ok(instance) => {
            log::info!(
                "Read {} download links from {}.",
                instance.len(),
                download_links_path.display()
            );
            instance
        }
    };

    for link in links {
        log::info!("Download {} to {}.", link.url, link.path);
        let destination_path = Path::new("./data/datasets/osm/").join(link.path);

        match download_file(&link.url, &destination_path, None).await {
            Err(why) => log::error!(
                "Failed to download {} to {}: {}",
                link.url,
                destination_path.display(),
                why
            ),
            Ok(human_readable_size) => log::info!(
                "Finished downloading {} ({})",
                destination_path.display(),
                human_readable_size
            ),
        }
    }
}
