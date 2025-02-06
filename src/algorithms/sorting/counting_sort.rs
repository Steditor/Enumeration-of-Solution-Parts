/// CountingSort as described in CLRS 4th edition 8.2, with a function to determine the sort key
pub fn counting_sort_by<T, F>(values: &[T], to_key: F, max_key: usize) -> Vec<T>
where
    T: Copy,
    F: Fn(&T) -> usize,
{
    let mut counts = vec![0; max_key + 1];
    for val in values.iter().as_ref() {
        counts[to_key(val)] += 1;
    }
    for i in 1..=max_key {
        counts[i] += counts[i - 1]
    }

    let mut output = vec![values[0]; values.len()];
    for val in values.iter().rev() {
        let key = to_key(val);
        output[counts[key] - 1] = *val;
        counts[key] -= 1;
    }

    output
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::counting_sort_by;

    /// Example in CLRS 4th edition figure 8.2
    const CLRS_8_2: [u8; 8] = [2, 5, 3, 0, 2, 3, 0, 3];
    /// Solution for CLRS 4th edition figure 8.2
    const CLRS_8_2_SOLUTION: [u8; 8] = [0, 0, 2, 2, 3, 3, 3, 5];

    #[test]
    fn test_clrs_8_2() {
        let sorted = counting_sort_by(&CLRS_8_2, |x| *x as usize, 5);

        assert_eq!(sorted, CLRS_8_2_SOLUTION);
    }

    #[test]
    fn test_stability() {
        let input = vec![
            (0, 2),
            (1, 5),
            (2, 3),
            (3, 0),
            (4, 2),
            (5, 3),
            (6, 0),
            (7, 3),
        ];
        let sorted = counting_sort_by(&input, |(_, v)| *v, 5);

        assert_eq!(
            sorted,
            vec![
                (3, 0),
                (6, 0),
                (0, 2),
                (4, 2),
                (2, 3),
                (5, 3),
                (7, 3),
                (1, 5)
            ]
        );
    }
}
