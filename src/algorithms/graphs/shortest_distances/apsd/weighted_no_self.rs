use num::Unsigned;

use crate::{
    data_structures::{
        graphs::{EdgeWeight, Graph},
        Index,
    },
    experiments::ExperimentAlgorithm,
};

use super::AlgorithmType;

pub const fn algorithm_enum_dijkstra<G, I, EW>() -> AlgorithmType<G, I, EW>
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    ExperimentAlgorithm::EnumerationAlgorithm("apsd-enum-dijkstra-no-self", |graph| {
        enumerate::prepare_enumeration(graph)
    })
}

mod enumerate {
    use num::Unsigned;

    use crate::{
        algorithms::{
            graphs::shortest_distances::{sssd::weighted::dijkstra, ShortestDistancePartial},
            sorting::counting_sort::counting_sort_by,
        },
        data_structures::{
            graphs::{Direction, EdgeWeight, Graph},
            Index,
        },
        experiments::PreparedEnumerationAlgorithm,
    };

    pub fn prepare_enumeration<G, I, EW>(
        graph: &G,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, EW>>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        let vertices_by_degree = counting_sort_by(
            &graph.vertices().collect::<Vec<_>>(),
            |v| graph.degree(*v, Direction::OUT).index(),
            graph.num_vertices().index() - 1,
        );

        let trivial_iterator = Box::new(
            vertices_by_degree
                .into_iter()
                .flat_map(|u| trivial_for(graph, u)),
        );

        let extension_iterator = Box::new(
            graph
                .vertices()
                .flat_map(|u| solution_row_iterator_for(graph, u)),
        );

        Box::new(trivial_iterator.chain(extension_iterator))
    }

    fn trivial_for<G, I, EW>(
        graph: &G,
        u: I,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, EW>>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        // no neighbors?
        if graph.degree(u, Direction::OUT) == I::zero() {
            // all other vertices are not reachable and thus a trivial solution part
            Box::new(
                graph
                    .vertices()
                    .filter(move |v| *v != u) // no-self
                    .map(move |v| (u, v, None)),
            )
        } else {
            // There are edges, the shortest edge is a solution part
            let (min_v, min_d) = graph
                .adjacencies(u, Direction::OUT)
                .min_by_key(|(_, d)| *d)
                .expect("We know that degree is > 0.");
            Box::new(std::iter::once((u, min_v, Some(min_d))))
        }
    }

    fn solution_row_iterator_for<G, I, EW>(
        graph: &G,
        u: I,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, EW>>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        // out-isolated vertex?
        if graph.degree(u, Direction::OUT) == I::zero() {
            // all solutions (non-reachable) already emitted in CAP
            Box::new(std::iter::empty())
        } else {
            // compute the whole solution row
            let solution_row = dijkstra(graph, u);
            // determine the solution part emitted in the CAP
            let (min_v, _) = graph
                .adjacencies(u, Direction::OUT)
                .min_by_key(|(_, d)| *d)
                .expect("We know that degree is > 0.");

            Box::new(
                graph
                    .vertices()
                    .filter(move |v| *v != u && *v != min_v) // no-self and not the trivial solution
                    .map(move |v| (u, v, solution_row[v.index()])),
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::data_structures::{graphs::DirectedAdjacencyArrayGraph, Matrix};

    use super::*;

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
    const SOLUTION_NONNEGATIVE_CLRS_23_4: [Option<u32>; 25] = [
        None, // Some(0),
        Some(3),
        Some(8),
        Some(4),
        Some(4),
        Some(3),
        None, // Some(0),
        Some(6),
        Some(1),
        Some(7),
        Some(7),
        Some(4),
        None, // Some(0),
        Some(5),
        Some(11),
        Some(2),
        Some(5),
        Some(5),
        None, // Some(0),
        Some(6),
        Some(8),
        Some(11),
        Some(11),
        Some(6),
        None, // Some(0),
    ];

    fn directed_nonnegative_crls_23_4() -> DirectedAdjacencyArrayGraph<u32, u32> {
        DirectedAdjacencyArrayGraph::new_with_edge_data(5, &EDGES_NONNEGATIVE_CLRS_23_4)
    }

    #[test]
    fn test_dijkstra_enumeration_nonnegative() {
        let graph = directed_nonnegative_crls_23_4();
        let mut distances = Matrix::new_square(graph.num_vertices().index());
        let mut done_parts = Matrix::<bool>::new_square(graph.num_vertices().index());
        let mut num_parts = 0;

        for (u, v, d) in enumerate::prepare_enumeration(&graph) {
            assert!(!done_parts[(u.index(), v.index())]);
            assert!(u != v); // no-self

            done_parts[(u.index(), v.index())] = true;
            num_parts += 1;

            distances[(u.index(), v.index())] = d;
        }
        // all non-self distances are n^2 - n
        assert_eq!(
            num_parts,
            graph.num_vertices().pow(2) - graph.num_vertices()
        );
        assert_eq!(
            distances,
            Matrix::new_square_from(&SOLUTION_NONNEGATIVE_CLRS_23_4)
        );
    }
}
