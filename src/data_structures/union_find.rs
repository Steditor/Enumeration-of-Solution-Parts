use super::{
    graphs::{Adjacency, Forest, Graph},
    Index,
};

/// Interface of a Union-Find data structure
pub trait DisjointSet<I: Index> {
    fn new_with_size(size: I) -> Self
    where
        Self: Sized;

    fn elements(&self) -> I::IndexIterator;

    fn find(&mut self, x: I) -> I;
    fn is_same(&mut self, x: I, y: I) -> bool {
        self.find(x) == self.find(y)
    }

    fn union(&mut self, x: I, y: I);

    fn sets(&mut self) -> Vec<Vec<I>>;
}

/// Union-Find data structure implemented as disjoint-set forest.
#[derive(Debug)]
pub struct UnionFind<I: Index>(Forest<I>);

impl<I: Index> DisjointSet<I> for UnionFind<I> {
    fn new_with_size(size: I) -> Self
    where
        Self: Sized,
    {
        Self(Forest::new_isolated_vertices(size))
    }

    fn elements(&self) -> <I as Index>::IndexIterator {
        I::zero().range(self.0.num_vertices())
    }

    /// Find with path halfing.
    ///
    /// As published in
    /// T. P. van der Weide,
    /// “Datastructures: An Axiomatic Approach and the Use of Binomial Trees in Developing and Analyzing Algorithms,”
    /// Mathematisch Centrum, Amsterdam, 1980.
    fn find(&mut self, x: I) -> I {
        let mut u = x;
        let mut v = self.parent_of(u);
        let mut w = self.parent_of(v);
        while v != w {
            self.set_parent(u, w);
            u = w;
            v = self.parent_of(u);
            w = self.parent_of(v);
        }
        w
    }

    fn union(&mut self, x: I, y: I) {
        // find set representatives
        let x = self.find(x);
        let y = self.find(y);

        // already the same set; nothing to do here
        if x == y {
            return;
        }

        self.set_parent(x, y);
    }

    /// Get a vector of all disjoint sets.
    ///
    /// As side effect this will flatten-out the Union-Find forest via [Self::flatten].
    fn sets(&mut self) -> Vec<Vec<I>> {
        self.flatten();

        let mut sets = vec![Vec::<I>::new(); self.0.num_vertices().index()];
        for x in self.0.vertices() {
            sets[self.parent_of(x).index()].push(x);
        }

        sets.into_iter().filter(|s| !s.is_empty()).collect()
    }
}

impl<I: Index> UnionFind<I> {
    #[inline]
    fn parent_of(&self, x: I) -> I {
        self.0[x.index()].map_or(x, |a| a.sink())
    }

    #[inline]
    pub fn is_same_parent(&self, x: I, y: I) -> bool {
        self.parent_of(x) == self.parent_of(y)
    }

    #[inline]
    fn set_parent(&mut self, child: I, new_parent: I) {
        self.0[child.index()] = Some((new_parent, ()));
    }

    /// Set each parent to the root of the respective set.
    pub fn flatten(&mut self) {
        // re-usable list of entries in a parent-chain
        let mut chain = Vec::with_capacity(self.0.num_vertices().index());

        for x in self.0.vertices() {
            chain.clear();
            let mut u = x;
            let mut v = self.parent_of(u);
            // traverse the parent-chain of x up to its root
            while u != v {
                chain.push(u);
                u = v;
                v = self.parent_of(u);
            }
            // have all entries of the chain point to the root
            for &entry in chain.iter() {
                self.set_parent(entry, v);
            }
        }
    }
}

/// Union-Find data structure implemented as disjoint-set forest with ranks.
#[derive(Debug)]
pub struct RankedUnionFind<I: Index> {
    forest: UnionFind<I>,
    ranks: Box<[I]>,
}

impl<I: Index> DisjointSet<I> for RankedUnionFind<I> {
    /// Create a new disjoint-set forest with `size` individual elements.
    fn new_with_size(size: I) -> Self
    where
        Self: Sized,
    {
        Self {
            forest: UnionFind::new_with_size(size),
            ranks: vec![I::zero(); size.index()].into(),
        }
    }

    fn elements(&self) -> <I as Index>::IndexIterator {
        self.forest.elements()
    }

    fn find(&mut self, x: I) -> I {
        self.forest.find(x)
    }

    /// Union by rank.
    ///
    /// As described in CRLS 3rd edition 21.3
    fn union(&mut self, x: I, y: I) {
        // find set representatives
        let x = self.find(x);
        let y = self.find(y);

        // already the same set; nothing to do here
        if x == y {
            return;
        }

        let rank_x = self.rank_of(x);
        let rank_y = self.rank_of(y);

        // link
        if rank_x > rank_y {
            self.forest.set_parent(y, x);
        } else {
            self.forest.set_parent(x, y);
            if rank_x == rank_y {
                self.set_rank(y, rank_y + I::one());
            }
        }
    }

    fn sets(&mut self) -> Vec<Vec<I>> {
        self.forest.sets()
    }
}

impl<I: Index> RankedUnionFind<I> {
    #[inline]
    fn rank_of(&self, x: I) -> I {
        self.ranks[x.index()]
    }

    #[inline]
    fn set_rank(&mut self, x: I, new_rank: I) {
        self.ranks[x.index()] = new_rank;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // connected component example in Figure 21.1 of CRLS 3rd edition
    #[test]
    fn test_crls_21_1() {
        let mut sets = RankedUnionFind::new_with_size(10);

        sets.union(1, 3);
        sets.union(4, 6);
        sets.union(0, 2);
        sets.union(7, 8);
        sets.union(0, 1);
        sets.union(4, 5);
        sets.union(1, 2);

        assert_eq!(
            sets.sets(),
            [vec![0, 1, 2, 3], vec![4, 5, 6], vec![7, 8], vec![9]]
        );
    }

    // connected component example in Figure 21.1 of CRLS 3rd edition
    #[test]
    fn test_flatten_crls_21_1() {
        let mut sets = UnionFind::new_with_size(10);

        sets.union(1, 3);
        sets.union(4, 6);
        sets.union(0, 2);
        sets.union(7, 8);
        sets.union(0, 1);
        sets.union(4, 5);
        sets.union(1, 2);

        assert_eq!(
            **sets.0,
            [
                Some((2, ())),
                Some((3, ())),
                Some((3, ())),
                None,
                Some((6, ())),
                None,
                Some((5, ())),
                Some((8, ())),
                None,
                None
            ]
        );
        sets.flatten();
        assert_eq!(
            **sets.0,
            [
                Some((3, ())),
                Some((3, ())),
                Some((3, ())),
                None,
                Some((5, ())),
                None,
                Some((5, ())),
                Some((8, ())),
                None,
                None
            ]
        );
    }
}
