use std::fmt::Debug;

use num::Zero;

use crate::{
    algorithms::graphs::shortest_distances::ShortestDistancePartial,
    data_structures::{
        graphs::{DirectedAdjacencyArrayGraph, Graph, UndirectedAdjacencyArrayGraph},
        Index, Matrix,
    },
};

fn gte_with_none_infty<T: Ord>(a: &Option<T>, b: &Option<T>) -> bool {
    match (a, b) {
        (None, None) => true,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(a), Some(b)) => a >= b,
    }
}

pub fn check_enumeration_result<I, D>(
    actual: &[ShortestDistancePartial<I, D>],
    expected: &Matrix<Option<D>>,
    require_no_self: bool,
    require_sorted: bool,
) where
    I: Index,
    D: Default + Copy + Zero + Ord + Debug,
{
    let mut distances = Matrix::new_square(expected.num_cols());
    let mut done_parts = Matrix::<bool>::new_square(expected.num_cols());
    let mut last_distance = Some(D::zero());

    for (u, v, d) in actual.iter().copied() {
        // no duplicate solution parts:
        assert!(
            !done_parts[(u.index(), v.index())],
            "Duplicate part ({},{}).",
            u.index(),
            v.index()
        );
        done_parts[(u.index(), v.index())] = true;

        // correct solution:
        assert_eq!(
            expected[(u.index(), v.index())],
            d,
            "Wrong distance: ({},{}) should be {:?}, was {:?}.",
            u.index(),
            v.index(),
            expected[(u.index(), v.index())],
            d,
        );
        distances[(u.index(), v.index())] = d;

        // correct order, if required:
        if require_sorted {
            assert!(
                gte_with_none_infty(&d, &last_distance),
                "Wrong order: ({},{}) with distance {:?} came after a part with distance {:?}.",
                u.index(),
                v.index(),
                d,
                last_distance
            );
            last_distance = d;
        }

        // no self-distance, if required:
        if require_no_self {
            assert!(
                u != v,
                "No-self requirement violated for ({},{})",
                u.index(),
                v.index(),
            );
        }
    }

    // no missing solution parts:
    let mut expected_num = expected.num_cols().pow(2);
    if require_no_self {
        expected_num -= expected.num_cols();
    }
    assert_eq!(
        actual.len(),
        expected_num,
        "Expected {} solution parts, but {} were given.",
        expected_num,
        actual.len()
    );
}

pub fn undirected_sample() -> UndirectedAdjacencyArrayGraph<u32> {
    UndirectedAdjacencyArrayGraph::new(5, &SAMPLE_EDGES)
}

pub fn undirected_sample_solution() -> Matrix<Option<u32>> {
    Matrix::new_square_from(&SAMPLE_SOLUTION_UNDIRECTED)
}

pub fn directed_sample() -> DirectedAdjacencyArrayGraph<u32> {
    DirectedAdjacencyArrayGraph::new(5, &SAMPLE_EDGES)
}

pub fn directed_sample_solution() -> Matrix<Option<u32>> {
    Matrix::new_square_from(&SAMPLE_SOLUTION_DIRECTED)
}

const SAMPLE_EDGES: [(u32, u32); 3] = [(0, 1), (1, 2), (3, 4)];

const SAMPLE_SOLUTION_UNDIRECTED: [Option<u32>; 25] = [
    Some(0),
    Some(1),
    Some(2),
    None,
    None,
    Some(1),
    Some(0),
    Some(1),
    None,
    None,
    Some(2),
    Some(1),
    Some(0),
    None,
    None,
    None,
    None,
    None,
    Some(0),
    Some(1),
    None,
    None,
    None,
    Some(1),
    Some(0),
];
const SAMPLE_SOLUTION_DIRECTED: [Option<u32>; 25] = [
    Some(0),
    Some(1),
    Some(2),
    None,
    None,
    None,
    Some(0),
    Some(1),
    None,
    None,
    None,
    None,
    Some(0),
    None,
    None,
    None,
    None,
    None,
    Some(0),
    Some(1),
    None,
    None,
    None,
    None,
    Some(0),
];

pub fn directed_crls_23_4() -> DirectedAdjacencyArrayGraph<u32, i32> {
    DirectedAdjacencyArrayGraph::new_with_edge_data(5, &EDGES_CLRS_23_4)
}

pub fn directed_crls_23_4_solution() -> Matrix<Option<i32>> {
    Matrix::new_square_from(&SOLUTION_CLRS_23_4.map(Some))
}

pub fn directed_nonnegative_crls_23_4() -> DirectedAdjacencyArrayGraph<u32, u32> {
    DirectedAdjacencyArrayGraph::new_with_edge_data(5, &EDGES_NONNEGATIVE_CLRS_23_4)
}

pub fn directed_nonnegative_crls_23_4_solution() -> Matrix<Option<u32>> {
    Matrix::new_square_from(&SOLUTION_NONNEGATIVE_CLRS_23_4.map(Some))
}

/// Example graph from CLRS 4th edition Figure 23.4
const EDGES_CLRS_23_4: [(u32, u32, i32); 9] = [
    (0, 1, 3),
    (0, 2, 8),
    (0, 4, -4),
    (1, 3, 1),
    (1, 4, 7),
    (2, 1, 4),
    (3, 0, 2),
    (3, 2, -5),
    (4, 3, 6),
];
const SOLUTION_CLRS_23_4: [i32; 25] = [
    0, 1, -3, 2, -4, 3, 0, -4, 1, -1, 7, 4, 0, 5, 3, 2, -1, -5, 0, -2, 8, 5, 1, 6, 0,
];

/// Adjusted non-negative example graph from CLRS 4th edition Figure 23.4
const EDGES_NONNEGATIVE_CLRS_23_4: [(u32, u32, u32); 9] = [
    (0, 1, 3),
    (0, 2, 8),
    (0, 4, 4),
    (1, 3, 1),
    (1, 4, 7),
    (2, 1, 4),
    (3, 0, 2),
    (3, 2, 5),
    (4, 3, 6),
];
const SOLUTION_NONNEGATIVE_CLRS_23_4: [u32; 25] = [
    0, 3, 8, 4, 4, 3, 0, 6, 1, 7, 7, 4, 0, 5, 11, 2, 5, 5, 0, 6, 8, 11, 11, 6, 0,
];
