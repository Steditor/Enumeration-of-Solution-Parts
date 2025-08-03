use std::{fmt::Debug, ops::Add};

use num::Zero;

use super::Index;

macro_rules! auto_impl {
    ($trait:ty, $($type:ty)*) => ($(
        impl $trait for $type {}
    )*)
}

pub trait EdgeData: Copy + Clone + Default + Debug + 'static {}
auto_impl!(EdgeData, () u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 f32 f64 usize);

pub trait EdgeWeight: EdgeData + Ord + Add<Self> + Zero {}
auto_impl!(EdgeWeight, u8 i8 u16 i16 u32 i32 u64 i64 u128 i128);

/// An edge of a vertex in a graph.
///
/// The edge has two endpoints and potentially data attached, such as an edge weight.
pub trait Edge<I: Index, ED: EdgeData>: Copy + Clone {
    fn source(&self) -> I;
    fn sink(&self) -> I;
    fn data(&self) -> ED;

    fn other(&self, vertex: I) -> I {
        if self.source() == vertex {
            self.sink()
        } else {
            self.source()
        }
    }

    fn is_same_undirected(&self, other: &Self) -> bool {
        (self.source() == other.source() || self.source() == other.sink())
            && self.sink() == other.other(self.source())
    }
}

impl<I: Index, ED: EdgeData> Edge<I, ED> for (I, I, ED) {
    fn source(&self) -> I {
        self.0
    }

    fn sink(&self) -> I {
        self.1
    }

    fn data(&self) -> ED {
        self.2
    }
}

impl<I: Index, ED: EdgeData> Edge<I, ED> for &(I, I, ED) {
    fn source(&self) -> I {
        self.0
    }

    fn sink(&self) -> I {
        self.1
    }

    fn data(&self) -> ED {
        self.2
    }
}

impl<I: Index> Edge<I, ()> for (I, I) {
    fn source(&self) -> I {
        self.0
    }

    fn sink(&self) -> I {
        self.1
    }

    fn data(&self) {}
}

impl<I: Index> Edge<I, ()> for &(I, I) {
    fn source(&self) -> I {
        self.0
    }

    fn sink(&self) -> I {
        self.1
    }

    fn data(&self) {}
}

/// An adjacency of a vertex in a graph.
///
/// An adjacency is stored in form of the 'other' endpoint of an edge.
/// It potentially has edge data attached, such as an edge weight.
pub trait Adjacency<I: Index, ED: EdgeData> {
    fn sink(&self) -> I;
    fn data(&self) -> ED;
}

impl<I: Index, ED: EdgeData> Adjacency<I, ED> for (I, ED) {
    fn sink(&self) -> I {
        self.0
    }

    fn data(&self) -> ED {
        self.1
    }
}

impl<I: Index, ED: EdgeData> Adjacency<I, ED> for &(I, ED) {
    fn sink(&self) -> I {
        self.0
    }

    fn data(&self) -> ED {
        self.1
    }
}

impl<I: Index> Adjacency<I, ()> for I {
    fn sink(&self) -> I {
        *self
    }

    fn data(&self) {}
}

impl<I: Index> Adjacency<I, ()> for &I {
    fn sink(&self) -> I {
        **self
    }

    fn data(&self) {}
}

/// An edge direction relative to a vertex.
#[derive(Copy, Clone)]
pub enum Direction {
    OUT,
    IN,
}

impl Direction {
    /// Gets the vertex of the edge that is identified with this direction.
    /// `OUT` gives the source vertex of the edge, `IN` the target vertex.
    pub fn vertex<I: Index, ED: EdgeData>(&self, e: &impl Edge<I, ED>) -> I {
        match self {
            Direction::OUT => e.source(),
            Direction::IN => e.sink(),
        }
    }

    /// Gets the vertex of the edge that is *not* identified with this direction.
    /// `OUT` gives the target vertex of the edge, `IN` the source vertex.
    pub fn other<I: Index, ED: EdgeData>(&self, e: &impl Edge<I, ED>) -> I {
        match self {
            Direction::OUT => e.sink(),
            Direction::IN => e.source(),
        }
    }
}
