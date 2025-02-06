use prim::IncrementalPrim;

use super::*;

use crate::{
    data_structures::graphs::{Edge, Graph, UndirectedEdgeListGraph},
    helpers::assert_same_elements,
};

/// MST examples in Figure 23.4/23.5 of CRLS 3rd edition
const CRLS_MST_EDGES: [(u32, u32, u8); 14] = [
    (0, 1, 4),
    (0, 7, 8),
    (1, 2, 8),
    (1, 7, 11),
    (2, 3, 7),
    (2, 5, 4),
    (2, 8, 2),
    (3, 4, 9),
    (3, 5, 14),
    (4, 5, 10),
    (5, 6, 2),
    (6, 7, 1),
    (6, 8, 6),
    (7, 8, 7),
];

#[test]
fn test_kruskal_crls() {
    let graph = UndirectedEdgeListGraph::new_with_edge_data(9, &CRLS_MST_EDGES);

    let mst = Kruskal::mst_for(&graph);

    assert_eq!(mst.edges().map(|e| e.data()).sum::<u8>(), 37);

    assert_same_elements(
        mst.edges(),
        [
            (6, 7, 1),
            (8, 2, 2),
            (5, 6, 2),
            (1, 0, 4),
            (2, 5, 4),
            (3, 2, 7),
            (7, 0, 8),
            (4, 3, 9),
        ],
    );
}

#[test]
fn test_boruvka_crls() {
    let graph = UndirectedEdgeListGraph::new_with_edge_data(9, &CRLS_MST_EDGES);

    let mst = Boruvka::mst_for(&graph);

    assert_eq!(mst.edges().map(|e| e.data()).sum::<u8>(), 37);

    assert_same_elements(
        mst.edges(),
        [
            (1, 0, 4),
            (8, 2, 2),
            (3, 2, 7),
            (4, 3, 9),
            (5, 6, 2),
            (6, 7, 1),
            (7, 0, 8),
            (2, 5, 4),
        ],
    );
}

#[test]
fn test_prim_crls() {
    let graph = UndirectedEdgeListGraph::new_with_edge_data(9, &CRLS_MST_EDGES);

    let mst = Prim::mst_for(&graph);

    assert_eq!(mst.edges().map(|e| e.data()).sum::<u8>(), 37);

    assert_same_elements(
        mst.edges(),
        [
            (1, 0, 4),
            (8, 2, 2),
            (3, 2, 7),
            (4, 3, 9),
            (5, 6, 2),
            (6, 7, 1),
            (7, 0, 8),
            (2, 5, 4),
        ],
    );
}

#[test]
fn test_enumeration_crls() {
    let graph = UndirectedEdgeListGraph::new_with_edge_data(9, &CRLS_MST_EDGES);

    let partials: Vec<_> = Incremental::<_, _, Boruvka>::enumerator_for(&graph).collect();

    assert_eq!(partials.iter().map(|e| e.data()).sum::<u8>(), 37);
    assert_same_elements(
        partials,
        [
            (0, 1, 4),
            (2, 8, 2),
            (2, 5, 4),
            (3, 2, 7),
            (4, 3, 9),
            (5, 6, 2),
            (6, 7, 1),
            (7, 0, 8),
        ],
    );
}

#[test]
fn test_prim_enumeration_crls() {
    let graph = UndirectedEdgeListGraph::new_with_edge_data(9, &CRLS_MST_EDGES);

    let partials: Vec<_> = IncrementalPrim::enumerator_for(&graph).collect();

    assert_eq!(partials.iter().map(|e| e.data()).sum::<u8>(), 37);
    assert_same_elements(
        partials,
        [
            (1, 0, 4),
            (7, 0, 8),
            (6, 7, 1),
            (5, 6, 2),
            (2, 5, 4),
            (8, 2, 2),
            (3, 2, 7),
            (4, 3, 9),
        ],
    );
}
