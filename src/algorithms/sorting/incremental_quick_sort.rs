use compare::{natural, Compare, Natural};

/// Incremental Quick Sort
///
/// This is an implementation of the incremental sorting algorithm by Paredes and Navarro \[1\].
///
/// Currently we use a fixed pivot element (last element in the array) without protection against bad pivot choice.
///
/// `IQS` is used as iterator:
///
/// ```
/// use exp_lib::algorithms::sorting::IQS;
///
/// let iqs = IQS::new(&[49, 81, 74, 12, 58, 92, 86, 33, 67, 18, 25, 37, 51, 63, 29, 41]);
/// let mut sorted = Vec::new();
/// for x in iqs {
///     println!("{}", x);
///     sorted.push(x);
/// }
/// assert_eq!(sorted, [12, 18, 25, 29, 33, 37, 41, 49, 51, 58, 63, 67, 74, 81, 86, 92]);
/// ```
///
/// \[1\] R. Paredes and G. Navarro, “Optimal Incremental Sorting,” in 2006 Proceedings of the Workshop on Algorithm Engineering and Experiments (ALENEX), in Proceedings. , Society for Industrial and Applied Mathematics, 2006, pp. 171–182. doi: [10.1137/1.9781611972863.16](https://doi.org/10.1137/1.9781611972863.16).
pub struct IQS<T, C>
where
    T: Copy,
    C: Compare<T>,
{
    pub a: Vec<T>,
    comparator: C,
    idx: usize,
    s: Vec<usize>,
}

impl<T> IQS<T, Natural<T>>
where
    T: Copy + Ord,
{
    pub fn new(elements: &[T]) -> Self {
        Self {
            s: vec![elements.len()],
            a: elements.to_vec(),
            comparator: natural(),
            idx: 0,
        }
    }
}

impl<T, C> IQS<T, C>
where
    T: Copy,
    C: Compare<T>,
{
    pub fn with_comparator(elements: &[T], comparator: C) -> Self {
        Self {
            s: vec![elements.len()],
            a: elements.to_vec(),
            comparator,
            idx: 0,
        }
    }
}

impl<T, C> Iterator for IQS<T, C>
where
    T: Copy,
    C: Compare<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.a.len() {
            // we're done; everything's sorted
            None
        } else {
            // keep sorting
            let mut top = *self.s.last().expect("stack can't run empty");

            // run until a[0..=idx] is sorted
            while self.idx != top {
                let pidx = partition(&mut self.a[self.idx..top], &self.comparator) + self.idx;
                self.s.push(pidx);
                top = pidx;
            }

            self.s.pop(); // pivot element has served its purpose
            self.idx += 1; // for next invocation of this function
            Some(self.a[top]) // return next sorted element
        }
    }
}

/// Partition slice `a` by a pivot and return the index of the pivot
///
/// Implementation is an adaptation of CLRS 4th edition / Chapter 7:
///
/// ```pseudo
/// PARTITION(A, p, r)
///   x = A[r]
///   i = p - 1
///   for j = p to r - 1
///     if A[j] <= x
///       i = i + 1
///       exchange A[i] with A[j]
///   exchange A[i+1] with A[r]
///   return i + 1
/// ```
/// Our changes:
/// - we use a slice instead of explicit bounds
/// - we partition by < instead of <=, so after the first partition phase elements left of pivot are strictly smaller
/// - i is one smaller than in CLRS version (left bound is 0 and usize can't hold -1)
/// - we add a second phase: after the initial partition we find and return the "most central" element that is "equal"
///   to pivot element to avoid worst-case recursion in case there are lots of equal elements.
///
/// # Panics
///
/// Panics if slice is empty.
fn partition<T, C: Compare<T>>(a: &mut [T], comparator: &C) -> usize
where
    T: Copy,
{
    // `pivot_value` is x in the CLRS partition pseudocode
    let pivot_value = *a.last().expect("slice must not be empty");

    // i tracks a possible pivot position
    let mut i = 0;
    // j looks for elements smaller than pivot that need to be left of the pivot
    for j in 0..a.len() - 1 {
        if comparator.compares_lt(&a[j], &pivot_value) {
            // element in position j is smaller than pivot
            // move it to the current possible pivot position
            a.swap(i, j);
            // next possible pivot position is now to the right of the swapped element
            i += 1;
        }
    }

    // swap pivot element to the confirmed pivot position i
    a.swap(i, a.len() - 1);
    // all elements left of i are smaller than pivot according to the comparator
    let leftmost_pivot_position = i;

    // There might be (a lot) of elements right of this pivot position that are
    // equal to the pivot in terms of the comparator.
    // this might result in a bad partition with only a constant number of elements
    // left of the pivot and a linear number of elements right of it.

    // we would like the pivot to be in the middle.
    let mid_position = a.len() / 2;

    // if the pivot is already right of or at the middle position, we don't want to go further to the right.
    if mid_position <= leftmost_pivot_position {
        return leftmost_pivot_position;
    }

    // pivot is in the left half, so let's try to find a better pivot position:
    // - let l be the leftmost_pivot_position as computed above
    // - assume the rightmost element equal to pivot is in position r
    // - we have to choose a pivot in the slice [l..=r]
    // - we do not want to go farther to the right than the `mid_position` m
    a[leftmost_pivot_position + 1..=mid_position]
        .iter()
        // if r < m, the following position search will find the larger element in position (r + 1);
        // but as our search slice starts at l+1, this actually returns Some(r + 1 - (l + 1)) as index
        .position(|y| comparator.compares_lt(&pivot_value, y))
        // we shift this index back to the actual index of r in a
        .map(|p| p + leftmost_pivot_position)
        // if r >= m, the search will yield None and we want to take the mid_position
        .unwrap_or(mid_position)
}

#[cfg(test)]
mod test {
    use super::*;

    const CRLS_7_1: [u32; 8] = [2, 8, 7, 1, 3, 5, 6, 4];
    const PAREDES_NAVARRO: [u32; 16] = [
        49, 81, 74, 12, 58, 92, 86, 33, 67, 18, 25, 37, 51, 63, 29, 41,
    ];

    #[test]
    fn test_partition_crls_7_1() {
        let mut a = CRLS_7_1;

        let pidx = partition(&mut a, &u32::cmp);

        assert_eq!(pidx, 3);
        assert_eq!(a, [2, 1, 3, 4, 7, 5, 6, 8]);
    }

    #[test]
    fn test_partition_equal_elements() {
        let mut a = vec![0, 0, 0, 0, 0];
        let pidx = partition(&mut a, &i32::cmp);

        assert_eq!(pidx, 2);
        assert_eq!(a, [0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_sort_crls_7_1() {
        let iqs = IQS::new(&CRLS_7_1);
        let sorted: Vec<u32> = iqs.collect();
        assert_eq!(sorted, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_sort_paredes_navarro() {
        let iqs = IQS::new(&PAREDES_NAVARRO);
        let sorted: Vec<u32> = iqs.collect();
        assert_eq!(
            sorted,
            [12, 18, 25, 29, 33, 37, 41, 49, 51, 58, 63, 67, 74, 81, 86, 92]
        );
    }
}
