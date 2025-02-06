use crate::{
    data_structures::{
        graphs::{EdgeData, Graph},
        Index,
    },
    experiments::ExperimentAlgorithm,
};

use super::AlgorithmType;

pub const fn algorithm_enum_bfs<G, I, ED>() -> AlgorithmType<G, I>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    ExperimentAlgorithm::EnumerationAlgorithm("apsd-enum-bfs-no-self", |graph| {
        enumerate::prepare_enumeration(graph)
    })
}

mod enumerate {
    use crate::{
        algorithms::graphs::shortest_distances::{sssd::unweighted::sssd, ShortestDistancePartial},
        data_structures::{
            graphs::{Direction, EdgeData, Graph},
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
        let trivial_iterator = Box::new(graph.vertices().flat_map(|u| trivial_for(graph, u)));

        let extension_iterator = Box::new(
            graph
                .vertices()
                .flat_map(|u| solution_row_iterator_for(graph, u)),
        );

        Box::new(trivial_iterator.chain(extension_iterator))
    }

    fn trivial_for<G, I, ED>(
        graph: &G,
        u: I,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, I>>
    where
        G: Graph<I, ED> + ?Sized,
        I: Index,
        ED: EdgeData,
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
            // There are edges, each edge is a solution part
            Box::new(
                graph
                    .neighbors(u, Direction::OUT)
                    .map(move |v| (u, v, Some(I::one()))),
            )
        }
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
        // out-isolated vertex?
        if graph.degree(u, Direction::OUT) == I::zero() {
            // all solutions (non-reachable) already emitted in CAP
            Box::new(std::iter::empty())
        } else {
            // compute the whole solution row
            let solution_row = sssd(graph, u);
            Box::new(
                graph
                    .vertices()
                    .filter(move |v| *v != u) // no-self
                    .map(move |v| (u, v, solution_row[v.index()]))
                    .filter(|(_, _, d)| !d.is_some_and(|d| d == I::one())), // single edges already emitted
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::algorithms::graphs::shortest_distances::apsd::tests::{
        check_enumeration_result, directed_sample, directed_sample_solution, undirected_sample,
        undirected_sample_solution,
    };

    use super::*;

    #[test]
    fn test_undirected_enumeration() {
        let graph = undirected_sample();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(&solution_parts, &undirected_sample_solution(), true, false);
    }

    #[test]
    fn test_directed_enumeration() {
        let graph = directed_sample();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(&solution_parts, &directed_sample_solution(), true, false);
    }
}
