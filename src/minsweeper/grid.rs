use minsweeper_rs::board::Point;
use std::iter::Flatten;
use std::ops::{Index, IndexMut};
use std::vec::IntoIter;

#[derive(Clone, Debug)]
pub struct Grid<E> {
    grid: Vec<Vec<E>>,
    width: usize,
    height: usize
}

impl<E> Grid<E> {
    const fn from_2d_vec(vec: Vec<Vec<E>>, width: usize, height: usize) -> Self<> {
        Self {
            grid: vec,
            width,
            height
        }
    }

    pub fn new(width: usize, height: usize, generator: impl Fn(Point) -> E) -> Self<> {
        let mut columns = vec![];
        for x in 0..width {
            let mut column = vec![];
            for y in 0..height {
                column.push(generator((x, y)));
            }
            columns.push(column);
        }
        Self::from_2d_vec(columns, width, height)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut E> {
        self.into_iter()
    }
}


impl<E> Index<Point> for Grid<E> {
    type Output = E;

    fn index(&self, index: Point) -> &Self::Output {
        &self.grid[index.0][index.1]
    }
}

impl<E> IndexMut<Point> for Grid<E> {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        &mut self.grid[index.0][index.1]
    }
}

impl<E> IntoIterator for Grid<E> {
    type Item = E;
    type IntoIter = Flatten<IntoIter<Vec<E>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.grid
                .into_iter()
                .flatten()
    }
}
impl<'a, E> IntoIterator for &'a Grid<E> {
    type Item = &'a E;
    type IntoIter = Flatten<std::slice::Iter<'a, Vec<E>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.grid
                .iter()
                .flatten()
    }
}

impl<'a, E> IntoIterator for &'a mut Grid<E> {
    type Item = &'a mut E;
    type IntoIter = Flatten<std::slice::IterMut<'a, Vec<E>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.grid
                .iter_mut()
                .flatten()
    }
}