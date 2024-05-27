use super::Index;

/// An edge direction relative to a vertex.
#[derive(Copy, Clone)]
pub enum Direction {
    OUT,
    IN,
}

impl Direction {
    /// Gets the vertex of the edge that is identified with this direction.
    /// `OUT` gives the source vertex of the edge, `IN` the target vertex.
    pub fn vertex<I: Index>(&self, (from, to): &(I, I)) -> I {
        match self {
            Direction::OUT => *from,
            Direction::IN => *to,
        }
    }

    /// Gets the vertex of the edge that is *not* identified with this direction.
    /// `OUT` gives the target vertex of the edge, `IN` the source vertex.
    pub fn other<I: Index>(&self, (from, to): &(I, I)) -> I {
        match self {
            Direction::OUT => *to,
            Direction::IN => *from,
        }
    }
}
