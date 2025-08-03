use std::{ops::ControlFlow, path::PathBuf, str::FromStr, time::Instant};

use crate::{
    algorithms::graphs::{
        search::{
            bfs::{bfs, BfsEvent},
            dfs::dfs_forest,
        },
        spanning_tree::undirected_weighted::{
            Boruvka, EnumMST, IncrementalPrim, Kruskal, MstAlgorithm, Prim,
        },
    },
    data_sets::{osm as osm_reader, DataSetReaderError},
    data_structures::{
        graphs::{Adjacency, Edge, Graph, UndirectedAdjacencyArrayGraph},
        Index,
    },
    io::json::write_json_to_file,
    visualization::helpers_osm as osm_vis,
};
use osm4routing::NodeId;
use serde::Serialize;

pub fn visualize() -> Result<(), DataSetReaderError> {
    let base_graph = read_graph()?;
    export_full_svg(&base_graph);
    visualize_prim(&base_graph);
    visualize_enum::<Prim>(&base_graph, "enum-prim");
    visualize_enum::<Kruskal>(&base_graph, "enum-kruskal");
    visualize_enum::<Boruvka>(&base_graph, "enum-boruvka");
    visualize_incremental_prim(&base_graph);
    Ok(())
}

struct BaseGraph {
    osm_nodes: Vec<osm4routing::Node>,
    osm_edges: Vec<osm4routing::Edge>,
    graph: UndirectedAdjacencyArrayGraph<u32, u32>,
}

fn read_graph() -> Result<BaseGraph, DataSetReaderError> {
    let in_path = "./data/datasets/osm/maps/north-america_us_michigan-latest.osm.pbf";
    let in_path = PathBuf::from_str(in_path).expect("Building the path can't fail.");

    log::info!("Read the graph from {in_path:?}");
    let osm_graph = osm_reader::OsmReader::read_from_file(
        &in_path,
        &osm_reader::OsmReaderOptions::new()
            .require_tag("highway", "motorway")
            .require_tag("highway", "motorway_link")
            .require_tag("highway", "trunk")
            .require_tag("highway", "trunk_link")
            .require_tag("highway", "primary")
            .require_tag("highway", "primary_link")
            .require_tag("highway", "secondary")
            .require_tag("highway", "secondary_link")
            .require_tag("highway", "tertiary")
            .require_tag("highway", "tertiary_link")
            .merge_ways(true),
    )?;
    log::info!("...done.");

    log::info!("Reduce to connected component.");
    let graph = osm_reader::OsmReader::to_weighted_undirected(&osm_graph)?;
    let mut reachable = vec![false; graph.num_vertices().index()];
    let _ = bfs(&graph, 0, &mut |e| {
        if let BfsEvent::Discovered(v) = e {
            reachable[v.index()] = true
        }
        ControlFlow::<()>::Continue(())
    });
    let (osm_nodes, osm_edges) = osm_graph;
    let node_translation = osm_reader::OsmReader::build_node_translation(&osm_nodes);
    let osm_nodes: Vec<_> = osm_nodes
        .into_iter()
        .filter(|n| reachable[node_translation.get(&n.id.0).expect("Node exists.").index()])
        .collect();
    let osm_edges: Vec<_> = osm_edges
        .into_iter()
        .filter(|e| {
            reachable[node_translation
                .get(&e.source.0)
                .expect("Node exists")
                .index()]
        })
        .collect();

    let osm_graph = (osm_nodes, osm_edges);
    let graph = osm_reader::OsmReader::to_weighted_undirected(&osm_graph)?;
    log::info!("...done.");
    let (osm_nodes, osm_edges) = osm_graph;

    Ok(BaseGraph {
        osm_nodes,
        osm_edges,
        graph,
    })
}

fn export_full_svg(base_graph: &BaseGraph) {
    let out_path = "./data/datasets/osm/maps/michigan-full.svg";
    log::info!("Saving full graph svg file to {out_path}");
    osm_vis::SvgWriter::for_graph(&base_graph.osm_nodes, &base_graph.osm_edges)
        .with_scale(1000.0)
        .write_to(out_path)
        .unwrap();
    log::info!("...done.");
}

fn visualize_prim(base_graph: &BaseGraph) {
    log::info!("Compute MST with Prim's algorithm.");
    let start = Instant::now();
    let mst = Prim::mst_for(&base_graph.graph);
    let prim_duration = start.elapsed().as_nanos();
    log::info!("...done in {prim_duration}ns.");

    log::info!("Mark prim MST edges.");
    let node_translation = osm_reader::OsmReader::build_node_translation(&base_graph.osm_nodes);

    let edge_classes = |(osm_from, osm_to): (NodeId, NodeId)| {
        let from = *node_translation.get(&osm_from.0).expect("Node exists.");
        let to = *node_translation.get(&osm_to.0).expect("Node exists.");

        if mst[from.index()].is_some_and(|(other, _)| other == to)
            || mst[to.index()].is_some_and(|(other, _)| other == from)
        {
            Some("mst".to_string())
        } else {
            None
        }
    };
    log::info!("...done.");

    let out_path = "./data/datasets/osm/maps/michigan-prim.svg";
    log::info!("Saving prim mst svg file to {out_path}");
    osm_vis::SvgWriter::for_graph(&base_graph.osm_nodes, &base_graph.osm_edges)
        .with_edge_classes(&edge_classes)
        .with_scale(1000.0)
        .with_extra_style(
            ".edges path{ stroke: none; } .edges .mst { stroke: #007a9e; stroke-width: 2; }"
                .to_string(),
        )
        .write_to(out_path)
        .unwrap();
    log::info!("...done.");
}

fn visualize_enum<BB: MstAlgorithm<u32, u32> + 'static>(
    base_graph: &BaseGraph,
    algorithm_name: impl AsRef<str>,
) {
    let algorithm_name = algorithm_name.as_ref();
    log::info!("Compute MST with {algorithm_name} algorithm.");
    let mut edge_list = Vec::with_capacity(base_graph.graph.num_vertices().index());

    let start = Instant::now();
    for edge in EnumMST::<_, _, BB>::enumerator_for(&base_graph.graph) {
        edge_list.push((edge, start.elapsed().as_nanos()));
    }
    let enum_duration = start.elapsed().as_nanos();
    log::info!("...done in {enum_duration}ns.");

    export_enum_visualization(base_graph, edge_list, algorithm_name);
}

fn visualize_incremental_prim(base_graph: &BaseGraph) {
    log::info!("Compute MST with incremental prim algorithm.");
    let mut edge_list = Vec::with_capacity(base_graph.graph.num_vertices().index());

    let start = Instant::now();
    for edge in IncrementalPrim::enumerator_for(&base_graph.graph) {
        edge_list.push((edge, start.elapsed().as_nanos()));
    }
    let enum_duration = start.elapsed().as_nanos();
    log::info!("...done in {enum_duration}ns.");

    export_enum_visualization(base_graph, edge_list, "incremental-prim");
}

fn export_enum_visualization(
    base_graph: &BaseGraph,
    edge_list: Vec<((u32, u32, u32), u128)>,
    algorithm_name: impl AsRef<str>,
) {
    let tree_edges: Vec<_> = edge_list
        .iter()
        .enumerate()
        .map(|(i, (edge, _))| (edge.source(), edge.sink(), i))
        .collect();
    let tree_graph = UndirectedAdjacencyArrayGraph::new_with_edge_data(
        base_graph.graph.num_vertices(),
        &tree_edges,
    );
    let mst = dfs_forest(&tree_graph);

    log::info!("Mark enum MST edges.");
    let node_translation = osm_reader::OsmReader::build_node_translation(&base_graph.osm_nodes);

    let edge_classes = |(osm_from, osm_to): (NodeId, NodeId)| {
        let from = *node_translation.get(&osm_from.0).expect("Node exists.");
        let to = *node_translation.get(&osm_to.0).expect("Node exists.");

        if mst[from.index()].is_some_and(|(other, _)| other == to) {
            Some(format!("mst edge-{}", mst[from.index()].unwrap().data()))
        } else if mst[to.index()].is_some_and(|(other, _)| other == from) {
            Some(format!("mst edge-{}", mst[to.index()].unwrap().data()))
        } else {
            None
        }
    };
    log::info!("...done.");
    let algorithm_name = algorithm_name.as_ref();
    let out_path = format!("./data/datasets/osm/maps/michigan-{algorithm_name}.svg");
    log::info!("Saving enum mst svg file to {out_path}");
    osm_vis::SvgWriter::for_graph(&base_graph.osm_nodes, &base_graph.osm_edges)
        .with_edge_classes(&edge_classes)
        .with_scale(1000.0)
        .with_extra_style(
            ".edges path{stroke: none} .edges .mst { stroke: #007a9e; stroke-width: 2; }"
                .to_string(),
        )
        .write_to(out_path)
        .unwrap();
    log::info!("...done.");

    let out_path = format!("./data/datasets/osm/maps/michigan-{algorithm_name}.json");
    log::info!("Saving edge timings to {out_path}");
    let edge_timings: Vec<_> = edge_list
        .iter()
        .enumerate()
        .map(|(i, (_, t))| EdgeTiming { i, t: *t })
        .collect();
    write_json_to_file(out_path, &edge_timings).unwrap();
    log::info!("...done.");
}

#[derive(Serialize)]
struct EdgeTiming {
    i: usize,
    t: u128,
}
