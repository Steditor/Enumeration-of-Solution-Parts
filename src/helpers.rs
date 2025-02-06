use std::fmt::Debug;

pub fn assert_same_elements<T>(a: impl IntoIterator<Item = T>, b: impl IntoIterator<Item = T>)
where
    T: Ord + PartialEq + Debug,
{
    let mut a: Vec<T> = a.into_iter().collect();
    a.sort();
    let mut b: Vec<T> = b.into_iter().collect();
    b.sort();

    assert_eq!(a, b)
}
