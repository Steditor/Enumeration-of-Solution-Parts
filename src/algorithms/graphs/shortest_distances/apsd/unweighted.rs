use std::ops::ControlFlow;

use crate::{
    algorithms::graphs::shortest_distances::{sssd::unweighted::sssd, ShortestDistancePartial},
    data_structures::{
        graphs::{EdgeData, Graph},
        Index, Matrix,
    },
    experiments::{CouldNotComputeError, ExperimentAlgorithm},
};

use super::AlgorithmType;

pub const fn algorithm_enum_bfs<G, I, ED>() -> AlgorithmType<G, I>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    ExperimentAlgorithm::EnumerationAlgorithm("apsd-enum-bfs", enumerate::prepare_enumeration)
}

mod enumerate {
    use crate::{
        algorithms::graphs::shortest_distances::{sssd::unweighted::sssd, ShortestDistancePartial},
        data_structures::{
            graphs::{EdgeData, Graph},
            Index,
        },
        experiments::PreparedEnumerationAlgorithm,
    };

    pub fn prepare_enumeration<G, I, ED>(
        graph: &G,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, I>>
    where
        G: Graph<I, ED> + ?Sized,
        I: Index,
        ED: EdgeData,
    {
        let trivial_iterator = Box::new(graph.vertices().map(|u| (u, u, Some(I::zero()))));

        let extension_iterator = Box::new(
            graph
                .vertices()
                .flat_map(|u| solution_row_iterator_for(graph, u)),
        );

        Box::new(trivial_iterator.chain(extension_iterator))
    }

    fn solution_row_iterator_for<G, I, ED>(
        graph: &G,
        u: I,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, I>>
    where
        G: Graph<I, ED> + ?Sized,
        I: Index,
        ED: EdgeData,
    {
        let solution_row = sssd(graph, u);
        Box::new(
            graph
                .vertices()
                .filter(move |v| *v != u) // self-distances already emitted
                .map(move |v| (u, v, solution_row[v.index()])),
        )
    }
}

pub const fn algorithm_bfs<G, I, ED>() -> AlgorithmType<G, I>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    ExperimentAlgorithm::TotalTimeAlgorithm("apsd-bfs", apsd_bfs)
}

pub fn apsd_bfs<G, I, ED>(graph: &G) -> Result<Matrix<Option<I>>, CouldNotComputeError>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
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
        for (v, d) in sssd(graph, u).iter().enumerate() {
            distances[(u.index(), v)] = *d;
        }
    }

    Ok(distances)
}

/// An apsd algorithm based on BFS that ignores the output.
///
/// This is needed because we can't possibly store the output for large input instances.
pub const fn algorithm_bfs_visitor<G, I, ED>() -> AlgorithmType<G, I>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    ExperimentAlgorithm::TotalTimeAlgorithm("apsd-bfs-visit", |graph| {
        apsd_bfs_visitor(graph, &mut |_| ControlFlow::<()>::Continue(()));
        // Return a dummy Matrix.
        Ok(Matrix::new_square(0))
    })
}

pub fn apsd_bfs_visitor<G, I, ED, B>(
    graph: &G,
    visitor: &mut impl FnMut(ShortestDistancePartial<I, I>) -> ControlFlow<B>,
) -> ControlFlow<B>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    for u in graph.vertices() {
        for (v, d) in sssd(graph, u).iter().enumerate() {
            visitor((u, I::new(v), *d))?;
        }
    }
    ControlFlow::Continue(())
}

#[cfg(test)]
mod test {
    use crate::algorithms::graphs::shortest_distances::apsd::tests::{
        check_enumeration_result, directed_sample, directed_sample_solution, undirected_sample,
        undirected_sample_solution,
    };

    use super::*;

    #[test]
    fn test_undirected() {
        let graph = undirected_sample();
        let distances = apsd_bfs(&graph).expect("This computation should work.");
        assert_eq!(distances, undirected_sample_solution(),);
    }

    #[test]
    fn test_directed() {
        let graph = directed_sample();
        let distances = apsd_bfs(&graph).expect("This computation should work.");
        assert_eq!(distances, directed_sample_solution());
    }

    #[test]
    fn test_undirected_enumeration() {
        let graph = undirected_sample();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(&solution_parts, &undirected_sample_solution(), false, false);
    }

    #[test]
    fn test_directed_enumeration() {
        let graph = directed_sample();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(&solution_parts, &directed_sample_solution(), false, false);
    }
}
