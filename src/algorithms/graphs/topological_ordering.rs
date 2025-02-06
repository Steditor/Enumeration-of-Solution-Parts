use core::fmt;
use std::{marker::PhantomData, ops::ControlFlow};

use crate::data_structures::{
    graphs::{DirectedGraph, Direction, EdgeData},
    Index,
};

use super::search::dfs::{dfs, DfsEvent, IDFS};

pub struct IterativeSourceRemoval<'a, DAG, I, ED>
where
    DAG: DirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    graph: &'a DAG,
    in_degrees: Vec<I>,
    sources: Vec<I>,
    num_ordered: I,
    _phantom: PhantomData<ED>,
}

impl<'a, DAG, I, ED> IterativeSourceRemoval<'a, DAG, I, ED>
where
    DAG: DirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    pub fn new(graph: &'a DAG) -> Self {
        let in_degrees: Vec<I> = graph
            .vertices()
            .map(|v| graph.degree(v, Direction::IN))
            .collect();

        let sources: Vec<I> = in_degrees
            .iter()
            .enumerate()
            .filter_map(|(v, deg)| {
                if *deg == I::zero() {
                    Some(I::new(v))
                } else {
                    None
                }
            })
            .collect();

        Self {
            graph,
            in_degrees,
            sources,
            num_ordered: I::zero(),
            _phantom: PhantomData,
        }
    }
}

impl<DAG, I, ED> Iterator for IterativeSourceRemoval<'_, DAG, I, ED>
where
    DAG: DirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    type Item = Result<I, HasCycles>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(source) = self.sources.pop() {
            for n in self.graph.neighbors(source, Direction::OUT) {
                self.in_degrees[n.index()] -= I::one();
                if self.in_degrees[n.index()] == I::zero() {
                    self.sources.push(n);
                }
            }
            self.num_ordered += I::one();
            Some(Ok(source))
        } else if self.num_ordered == self.graph.num_vertices() {
            None
        } else {
            Some(Err(HasCycles))
        }
    }
}

/// Compute a topological ordering with an incremental algorithm for DFS finishing times
pub fn idfs_finish_time<I: Index, ED: EdgeData>(
    graph: &impl DirectedGraph<I, ED>,
) -> Result<Vec<I>, HasCycles> {
    let mut order = vec![I::zero(); graph.num_vertices().index()];
    let mut index = order.len();

    let mut dfs = IDFS::new(graph.num_vertices());

    while let Some(e) = dfs.next(graph) {
        match e {
            DfsEvent::Finished(v) => {
                // topological ordering = vertices sorted by decreasing finish time
                index -= 1;
                order[index] = v;
            }
            DfsEvent::BackEdge(_, _, _) => {
                // DAGs have no back edges.
                return Err(HasCycles);
            }
            _ => (), // ignore other events
        }
    }

    Ok(order)
}

/// Compute a topological ordering with a recursive algorithm for DFS finishing times
pub fn dfs_finish_time<I: Index, ED: EdgeData>(
    graph: &impl DirectedGraph<I, ED>,
) -> Result<Vec<I>, HasCycles> {
    let mut order = vec![I::zero(); graph.num_vertices().index()];
    let mut index = order.len();

    match dfs(graph, &mut |e: DfsEvent<I, ED>| {
        match e {
            DfsEvent::Finished(v) => {
                // topological ordering = vertices sorted by decreasing finish time
                index -= 1;
                order[index] = v;
                ControlFlow::Continue(())
            }
            DfsEvent::BackEdge(_, _, _) => {
                // DAGs have no back edges.
                ControlFlow::Break(HasCycles)
            }
            _ => ControlFlow::Continue(()), // ignore other events
        }
    }) {
        ControlFlow::Continue(_) => Ok(order),
        ControlFlow::Break(err) => Err(err),
    }
}

#[derive(Debug, PartialEq)]
pub struct HasCycles;

impl fmt::Display for HasCycles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Graph is not a DAG and thus cannot be sorted topologically."
        )
    }
}
impl std::error::Error for HasCycles {}

#[cfg(test)]
mod test {
    use crate::data_structures::graphs::{DirectedAdjacencyArrayGraph, Graph};

    use super::*;

    // at the time of writing the test, this is the 5-node instance generated with
    // the random generator for DAGs with Taillard LCG (seed 42).
    const EDGES: [(u32, u32); 5] = [(0, 3), (1, 2), (3, 4), (3, 1), (4, 2)];
    const TOPO_ORDER: [u32; 5] = [0, 3, 1, 4, 2];

    const EDGES_WITH_CYCLE: [(u32, u32); 6] = [(0, 3), (1, 2), (2, 3), (3, 4), (3, 1), (4, 2)];

    #[test]
    fn test_iterative_source_removal() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(5, &EDGES);
        let order: Result<Vec<u32>, HasCycles> = IterativeSourceRemoval::new(&graph).collect();
        assert_eq!(order.unwrap(), TOPO_ORDER);
    }

    #[test]
    fn test_idfs_finish_time() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(5, &EDGES);
        let order: Result<Vec<u32>, HasCycles> = idfs_finish_time(&graph);
        assert_eq!(order.unwrap(), TOPO_ORDER);
    }

    #[test]
    fn test_dfs_finish_time() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(5, &EDGES);
        let order: Result<Vec<u32>, HasCycles> = dfs_finish_time(&graph);
        assert_eq!(order.unwrap(), TOPO_ORDER);
    }

    #[test]
    fn test_iterative_source_removal_with_cycle() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(5, &EDGES_WITH_CYCLE);
        let order: Result<Vec<u32>, HasCycles> = IterativeSourceRemoval::new(&graph).collect();
        assert!(order.is_err_and(|e| e == HasCycles));
    }

    #[test]
    fn test_idfs_finish_time_with_cycle() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(5, &EDGES_WITH_CYCLE);
        let order: Result<Vec<u32>, HasCycles> = idfs_finish_time(&graph);
        assert!(order.is_err_and(|e| e == HasCycles));
    }

    #[test]
    fn test_dfs_finish_time_with_cycle() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(5, &EDGES_WITH_CYCLE);
        let order: Result<Vec<u32>, HasCycles> = dfs_finish_time(&graph);
        assert!(order.is_err_and(|e| e == HasCycles));
    }
}
