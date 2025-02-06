use num::Unsigned;

use crate::{
    algorithms::graphs::shortest_distances::sssd::weighted::dijkstra,
    data_structures::{
        graphs::{EdgeWeight, Graph},
        Index, Matrix,
    },
    experiments::{CouldNotComputeError, ExperimentAlgorithm},
};

use super::AlgorithmType;

pub const fn algorithm_enum_dijkstra<G, I, EW>() -> AlgorithmType<G, I, EW>
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    ExperimentAlgorithm::EnumerationAlgorithm("apsd-enum-dijkstra", enumerate::prepare_enumeration)
}

mod enumerate {
    use num::Unsigned;

    use crate::{
        algorithms::graphs::shortest_distances::{
            sssd::weighted::dijkstra, ShortestDistancePartial,
        },
        data_structures::{
            graphs::{EdgeWeight, Graph},
            Index,
        },
        experiments::PreparedEnumerationAlgorithm,
    };

    pub fn prepare_enumeration<G, I, EW>(
        graph: &G,
    ) -> PreparedEnumerationAlgorithm<'_, ShortestDistancePartial<I, EW>>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        let trivial_iterator = Box::new(graph.vertices().map(|u| (u, u, Some(EW::zero()))));

        let extension_iterator = Box::new(
            graph
                .vertices()
                .flat_map(|u| solution_row_iterator_for(graph, u)),
        );

        Box::new(trivial_iterator.chain(extension_iterator))
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
        let solution_row = dijkstra(graph, u);
        Box::new(
            graph
                .vertices()
                .filter(move |v| *v != u) // self-distances already emitted
                .map(move |v| (u, v, solution_row[v.index()])),
        )
    }
}

pub const fn algorithm_dijkstra<G, I, EW>() -> AlgorithmType<G, I, EW>
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    ExperimentAlgorithm::TotalTimeAlgorithm("apsd-dijkstra", apsd_dijkstra)
}

pub fn apsd_dijkstra<G, I, EW>(graph: &G) -> Result<Matrix<Option<EW>>, CouldNotComputeError>
where
    G: Graph<I, EW> + ?Sized,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    let mut distances = match Matrix::try_new_square(graph.num_vertices().index()) {
        Ok(m) => m,
        Err(why) => {
            return Err(CouldNotComputeError {
                reason: why.to_string(),
            })
        }
    };

    for u in graph.vertices() {
        for (v, d) in dijkstra(graph, u).iter().enumerate() {
            distances[(u.index(), v)] = *d;
        }
    }

    Ok(distances)
}

pub const fn algorithm_floyd_warshall<G, I, EW>() -> AlgorithmType<G, I, EW>
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight,
{
    ExperimentAlgorithm::TotalTimeAlgorithm("apsd-fw", apsd_floyd_warshall)
}

/// An implementation of the Floyd-Warshall-Algorithm.
///
/// The algorithm can fail if rust cannot allocate enough memory to build a full distance matrix.
pub fn apsd_floyd_warshall<G, I, EW>(graph: &G) -> Result<Matrix<Option<EW>>, CouldNotComputeError>
where
    G: Graph<I, EW> + ?Sized,
    I: Index,
    EW: EdgeWeight,
{
    let mut distances = match Matrix::try_new_square(graph.num_vertices().index()) {
        Ok(m) => m,
        Err(why) => {
            return Err(CouldNotComputeError {
                reason: why.to_string(),
            })
        }
    };

    for v in graph.vertices() {
        distances[(v.index(), v.index())] = Some(EW::zero());
    }

    for (u, v, d) in graph.edges() {
        distances[(u.index(), v.index())] = Some(d);
    }

    for k in graph.vertices() {
        for i in graph.vertices() {
            for j in graph.vertices() {
                let d_ij = distances[(i.index(), j.index())];
                let d_ik = distances[(i.index(), k.index())];
                let d_kj = distances[(k.index(), j.index())];
                distances[(i.index(), j.index())] = match (d_ij, d_ik.zip(d_kj)) {
                    (None, Some((d_ik, d_kj))) => Some(d_ik + d_kj),
                    (x, None) => x,
                    (Some(d_ij), Some((d_ik, d_kj))) => Some(d_ij.min(d_ik + d_kj)),
                };
            }
        }
    }

    Ok(distances)
}

#[cfg(test)]
mod test {
    use crate::algorithms::graphs::shortest_distances::apsd::tests::{
        check_enumeration_result, directed_crls_23_4, directed_crls_23_4_solution,
        directed_nonnegative_crls_23_4, directed_nonnegative_crls_23_4_solution,
    };

    use super::*;

    #[test]
    fn test_floyd_warshall() {
        let graph = directed_crls_23_4();
        let distances = apsd_floyd_warshall(&graph).expect("This computation should work");
        assert_eq!(distances, directed_crls_23_4_solution(),);
    }

    #[test]
    fn test_floyd_warshall_nonnegative() {
        let graph = directed_nonnegative_crls_23_4();
        let distances = apsd_floyd_warshall(&graph).expect("This computation should work");
        assert_eq!(distances, directed_nonnegative_crls_23_4_solution(),);
    }

    #[test]
    fn test_dijkstra_nonnegative() {
        let graph = directed_nonnegative_crls_23_4();
        let distances = apsd_dijkstra(&graph).expect("This computation should work");
        assert_eq!(distances, directed_nonnegative_crls_23_4_solution(),);
    }

    #[test]
    fn test_dijkstra_enumeration_nonnegative() {
        let graph = directed_nonnegative_crls_23_4();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(
            &solution_parts,
            &directed_nonnegative_crls_23_4_solution(),
            false,
            false,
        );
    }
}
