use std::{
    collections::TryReserveError,
    fmt::Debug,
    ops::{Index, IndexMut},
};

use num::integer::Roots;

pub struct Matrix<T> {
    data: Vec<T>,
    num_cols: usize,
}

impl<T> Matrix<T> {
    /// Returns the number of rows of the matrix
    pub fn num_rows(&self) -> usize {
        self.data.len() / self.num_cols
    }

    /// Returns the number of columns of the matrix
    pub fn num_cols(&self) -> usize {
        self.num_cols
    }

    /// Iterate over all values in the matrix in row-major order
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Iterate over all (row, col, val)-entries in the matrix in row-major order
    ///
    /// ```
    /// use exp_lib::data_structures::Matrix;
    ///
    /// let matrix = Matrix::new_square_from(&[0, 1, 2, 3, 4, 5, 6, 7, 8]);
    /// assert_eq!(matrix.iter_matrix().collect::<Vec<_>>(), &[(0, 0, &0), (1, 0, &1), (2, 0, &2), (0, 1, &3), (1, 1, &4), (2, 1, &5), (0, 2, &6), (1, 2, &7), (2, 2, &8)]);
    /// ```
    pub fn iter_matrix(&self) -> MatrixIterator<T> {
        MatrixIterator::new(self)
    }
}

impl<T> Matrix<T>
where
    T: Default + Clone,
{
    /// Create a new matrix with the given dimensions and with the default value of `T` in each cell
    pub fn new_rect(num_rows: usize, num_cols: usize) -> Self {
        Matrix {
            data: vec![Default::default(); num_rows * num_cols],
            num_cols,
        }
    }

    /// Try to create a new matrix with the given dimensions and with the default value of `T` in each cell
    ///
    /// Will fail with `TryReserveError` if the requested matrix was too large to being allocated.
    pub fn try_new_rect(num_rows: usize, num_cols: usize) -> Result<Self, TryReserveError> {
        let mut data = Vec::try_with_capacity(num_rows * num_cols)?;
        data.resize(num_rows * num_cols, Default::default());
        Ok(Matrix { data, num_cols })
    }

    /// Create a new matrix with `len` rows and colums and with the default value of `T` in each cell
    pub fn new_square(len: usize) -> Self {
        Self::new_rect(len, len)
    }

    /// Try to create a new matrix with `len` rows and colums and with the default value of `T` in each cell
    ///
    /// Will fail with `TryReserveError` if the requested matrix was too large to being allocated.
    pub fn try_new_square(len: usize) -> Result<Self, TryReserveError> {
        Self::try_new_rect(len, len)
    }

    /// Create a new square matrix by copying the given data while inferring the number of rows and columns
    ///
    /// # Panics
    ///
    /// Panics if `data.len()` is not a square number
    pub fn new_square_from(data: &[T]) -> Self {
        let len = data.len().sqrt();
        if len * len != data.len() {
            panic!("Cannot create a square matrix with {} cells", data.len())
        }

        Matrix {
            data: data.to_vec(),
            num_cols: len,
        }
    }
}

/// Access the cell (row, column) of the matrix.
impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (row, column): (usize, usize)) -> &Self::Output {
        &self.data[row * self.num_cols + column]
    }
}

/// Access the cell (row, column) of the matrix mutably.
impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (row, column): (usize, usize)) -> &mut Self::Output {
        &mut self.data[row * self.num_cols + column]
    }
}

pub struct MatrixIterator<'a, T> {
    matrix: &'a Matrix<T>,
    pos: usize,
}

impl<'a, T> MatrixIterator<'a, T> {
    fn new(matrix: &'a Matrix<T>) -> Self {
        Self { matrix, pos: 0 }
    }
}

impl<'a, T> Iterator for MatrixIterator<'a, T> {
    type Item = (usize, usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.matrix.data.len() {
            let cur_pos = self.pos;
            self.pos += 1;
            Some((
                cur_pos % self.matrix.num_cols,
                cur_pos / self.matrix.num_cols,
                &self.matrix.data[cur_pos],
            ))
        } else {
            None
        }
    }
}

impl<T> Debug for Matrix<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matrix")
            .field("data", &self.data)
            .field("num_cols", &self.num_cols)
            .finish()
    }
}

impl<T> PartialEq<Matrix<T>> for Matrix<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Matrix<T>) -> bool {
        self.data == other.data && self.num_cols == other.num_cols
    }
}
