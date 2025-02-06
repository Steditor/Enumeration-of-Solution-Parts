pub use super::*;
use crate::helpers::assert_same_elements;

pub mod directed_weighted {
    use rstest::rstest;

    use super::*;

    pub fn edges() -> Box<[(u32, u32, u8)]> {
        Box::new([
            (2, 1, 1),
            (4, 1, 2),
            (2, 4, 3),
            (4, 3, 4),
            (3, 4, 5),
            (2, 3, 6),
            (3, 3, 7),
        ])
    }

    fn build<T: DirectedGraph<u32, u8>>() -> T {
        T::new_with_edge_data(6, &edges())
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_vertices(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.num_vertices(), 6);
        assert_same_elements(graph.vertices(), [0, 1, 2, 3, 4, 5]);
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_edges(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.num_edges(), 7);
        assert_same_elements(graph.edges(), edges().iter().copied());
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_out_degree(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.degree(0, Direction::OUT), 0);
        assert_eq!(graph.degree(1, Direction::OUT), 0);
        assert_eq!(graph.degree(2, Direction::OUT), 3);
        assert_eq!(graph.degree(3, Direction::OUT), 2);
        assert_eq!(graph.degree(4, Direction::OUT), 2);
        assert_eq!(graph.degree(5, Direction::OUT), 0);
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_out_neighbors(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.neighbors(0, Direction::OUT), []);
        assert_same_elements(graph.neighbors(1, Direction::OUT), []);
        assert_same_elements(graph.neighbors(2, Direction::OUT), [1, 4, 3]);
        assert_same_elements(graph.neighbors(3, Direction::OUT), [4, 3]);
        assert_same_elements(graph.neighbors(4, Direction::OUT), [1, 3]);
        assert_same_elements(graph.neighbors(5, Direction::OUT), []);
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_out_adjacencies(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.adjacencies(0, Direction::OUT), []);
        assert_same_elements(graph.adjacencies(1, Direction::OUT), []);
        assert_same_elements(
            graph.adjacencies(2, Direction::OUT),
            [(1, 1), (4, 3), (3, 6)],
        );
        assert_same_elements(graph.adjacencies(3, Direction::OUT), [(4, 5), (3, 7)]);
        assert_same_elements(graph.adjacencies(4, Direction::OUT), [(1, 2), (3, 4)]);
        assert_same_elements(graph.adjacencies(5, Direction::OUT), []);
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_in_degree(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.degree(0, Direction::IN), 0);
        assert_eq!(graph.degree(1, Direction::IN), 2);
        assert_eq!(graph.degree(2, Direction::IN), 0);
        assert_eq!(graph.degree(3, Direction::IN), 3);
        assert_eq!(graph.degree(4, Direction::IN), 2);
        assert_eq!(graph.degree(5, Direction::IN), 0);
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_in_neighbors(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.neighbors(0, Direction::IN), []);
        assert_same_elements(graph.neighbors(1, Direction::IN), [2, 4]);
        assert_same_elements(graph.neighbors(2, Direction::IN), []);
        assert_same_elements(graph.neighbors(3, Direction::IN), [2, 3, 4]);
        assert_same_elements(graph.neighbors(4, Direction::IN), [2, 3]);
        assert_same_elements(graph.neighbors(5, Direction::IN), []);
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_,_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_,_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_in_adjacencies(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.adjacencies(0, Direction::IN), []);
        assert_same_elements(graph.adjacencies(1, Direction::IN), [(2, 1), (4, 2)]);
        assert_same_elements(graph.adjacencies(2, Direction::IN), []);
        assert_same_elements(
            graph.adjacencies(3, Direction::IN),
            [(2, 6), (3, 7), (4, 4)],
        );
        assert_same_elements(graph.adjacencies(4, Direction::IN), [(2, 3), (3, 5)]);
        assert_same_elements(graph.adjacencies(5, Direction::IN), []);
    }
}

pub mod directed {
    use rstest::rstest;

    use super::*;

    pub fn edges() -> Box<[(u32, u32)]> {
        Box::new([(2, 1), (4, 1), (2, 4), (4, 3), (3, 4), (2, 3), (3, 3)])
    }

    fn build<T: DirectedGraph<u32>>() -> T {
        T::new(6, &edges())
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_>>())]
    pub fn test_vertices(#[case] graph: impl Graph<u32>) {
        assert_eq!(graph.num_vertices(), 6);
        assert_same_elements(graph.vertices(), [0, 1, 2, 3, 4, 5]);
    }

    #[rstest]
    #[case::directed_adjacency_array(build::<DirectedAdjacencyArrayGraph<_>>())]
    #[case::in_out_adjacency_arrays(build::<InOutAdjacencyArraysGraph<_>>())]
    #[case::directed_edge_list(build::<DirectedEdgeListGraph<_,_>>())]
    pub fn test_edges(#[case] graph: impl Graph<u32>) {
        assert_eq!(graph.num_edges(), 7);
        assert_same_elements(
            graph.edges(),
            edges()
                .iter()
                .map(|e| (e.source(), edge::Edge::sink(&e), ())),
        );
    }
}

pub mod undirected_weighted {
    use rstest::rstest;

    use super::*;

    pub fn edges() -> Box<[(u32, u32, u8)]> {
        Box::new([
            (2, 1, 1),
            (4, 1, 2),
            (2, 4, 3),
            (4, 3, 4),
            (3, 3, 5),
            (2, 3, 6),
        ])
    }

    fn build<T: UndirectedGraph<u32, u8>>() -> T {
        T::new_with_edge_data(6, &edges())
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_vertices(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.num_vertices(), 6);
        assert_same_elements(graph.vertices(), [0, 1, 2, 3, 4, 5]);
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_edges(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.num_edges(), 6);
        assert_same_elements(
            graph.edges(),
            [
                (2, 1, 1),
                (1, 2, 1),
                (4, 1, 2),
                (1, 4, 2),
                (2, 4, 3),
                (4, 2, 3),
                (4, 3, 4),
                (3, 4, 4),
                (3, 3, 5),
                (2, 3, 6),
                (3, 2, 6),
            ],
        );
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_out_degree(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.degree(0, Direction::OUT), 0);
        assert_eq!(graph.degree(1, Direction::OUT), 2);
        assert_eq!(graph.degree(2, Direction::OUT), 3);
        assert_eq!(graph.degree(3, Direction::OUT), 3);
        assert_eq!(graph.degree(4, Direction::OUT), 3);
        assert_eq!(graph.degree(5, Direction::OUT), 0);
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_out_neighbors(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.neighbors(0, Direction::OUT), []);
        assert_same_elements(graph.neighbors(1, Direction::OUT), [2, 4]);
        assert_same_elements(graph.neighbors(2, Direction::OUT), [1, 4, 3]);
        assert_same_elements(graph.neighbors(3, Direction::OUT), [2, 3, 4]);
        assert_same_elements(graph.neighbors(4, Direction::OUT), [1, 2, 3]);
        assert_same_elements(graph.neighbors(5, Direction::OUT), []);
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_out_adjacencies(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.adjacencies(0, Direction::OUT), []);
        assert_same_elements(graph.adjacencies(1, Direction::OUT), [(2, 1), (4, 2)]);
        assert_same_elements(
            graph.adjacencies(2, Direction::OUT),
            [(1, 1), (4, 3), (3, 6)],
        );
        assert_same_elements(
            graph.adjacencies(3, Direction::OUT),
            [(4, 4), (3, 5), (2, 6)],
        );
        assert_same_elements(
            graph.adjacencies(4, Direction::OUT),
            [(1, 2), (2, 3), (3, 4)],
        );
        assert_same_elements(graph.adjacencies(5, Direction::OUT), []);
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_in_degree(#[case] graph: impl Graph<u32, u8>) {
        assert_eq!(graph.degree(0, Direction::IN), 0);
        assert_eq!(graph.degree(1, Direction::IN), 2);
        assert_eq!(graph.degree(2, Direction::IN), 3);
        assert_eq!(graph.degree(3, Direction::IN), 3);
        assert_eq!(graph.degree(4, Direction::IN), 3);
        assert_eq!(graph.degree(5, Direction::IN), 0);
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_in_neighbors(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.neighbors(0, Direction::IN), []);
        assert_same_elements(graph.neighbors(1, Direction::IN), [2, 4]);
        assert_same_elements(graph.neighbors(2, Direction::IN), [1, 4, 3]);
        assert_same_elements(graph.neighbors(3, Direction::IN), [2, 3, 4]);
        assert_same_elements(graph.neighbors(4, Direction::IN), [1, 2, 3]);
        assert_same_elements(graph.neighbors(5, Direction::IN), []);
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_,_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_,_>>())]
    pub fn test_in_adjacencies(#[case] graph: impl Graph<u32, u8>) {
        assert_same_elements(graph.adjacencies(0, Direction::IN), []);
        assert_same_elements(graph.adjacencies(1, Direction::IN), [(2, 1), (4, 2)]);
        assert_same_elements(
            graph.adjacencies(2, Direction::IN),
            [(1, 1), (4, 3), (3, 6)],
        );
        assert_same_elements(
            graph.adjacencies(3, Direction::IN),
            [(4, 4), (3, 5), (2, 6)],
        );
        assert_same_elements(
            graph.adjacencies(4, Direction::IN),
            [(1, 2), (2, 3), (3, 4)],
        );
        assert_same_elements(graph.adjacencies(5, Direction::IN), []);
    }
}

pub mod undirected {
    use rstest::rstest;

    use super::*;

    pub fn edges() -> Box<[(u32, u32)]> {
        Box::new([(2, 1), (4, 1), (2, 4), (4, 3), (3, 3), (2, 3)])
    }

    fn build<T: UndirectedGraph<u32>>() -> T {
        T::new(6, &edges())
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_>>())]
    pub fn test_vertices(#[case] graph: impl Graph<u32>) {
        assert_eq!(graph.num_vertices(), 6);
        assert_same_elements(graph.vertices(), [0, 1, 2, 3, 4, 5]);
    }

    #[rstest]
    #[case::undirected_adjacency_array(build::<UndirectedAdjacencyArrayGraph<_>>())]
    #[case::undirected_edge_list(build::<UndirectedEdgeListGraph<_>>())]
    pub fn test_edges(#[case] graph: impl Graph<u32>) {
        assert_eq!(graph.num_edges(), 6);
        assert_same_elements(
            graph.edges(),
            [
                (2, 1, ()),
                (1, 2, ()),
                (4, 1, ()),
                (1, 4, ()),
                (2, 4, ()),
                (4, 2, ()),
                (4, 3, ()),
                (3, 4, ()),
                (3, 3, ()),
                (2, 3, ()),
                (3, 2, ()),
            ],
        );
    }
}
