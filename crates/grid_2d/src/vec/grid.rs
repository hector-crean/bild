use super::iter::*;

use std::mem;
use bevy::ecs::component::Component;
use bevy::reflect::Reflect;
use crate::position::Position;
use crate::pattern::*;
use crate::step::*;

/// 2D Grid, Position (0,0) is at the top left corner
#[derive(Debug, PartialEq, Reflect, Component)]
pub struct Grid<T> {
    pub(crate) items: Vec<T>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl<T: Default> Default for Grid<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            width: 0,
            height: 0,
        }
    }
}

impl<T: Clone> Grid<T> {
    /// Creates a new Grid with `default_value` as every value.
    /// # Panics
    /// * if width or height are zero
    pub fn new(width: usize, height: usize, default_value: T) -> Self {
        if width == 0 || height == 0 {
            panic!("width and height must be positive");
        }
        Self {
            width,
            height,
            items: vec![default_value; width * height],
        }
    }
}

impl<T: Default> Grid<T> {
    /// Returns the item at `pos` and leaves `T::Default()` in it's place,
    /// or `None` if `pos` is out of bounds.
    pub fn replace_default<P: Into<Position>>(&mut self, pos: P) -> Option<T> {
        let pos = pos.into();
        if self.is_bounds(pos) {
            let idx = self.translate(pos);
            let old = mem::take(&mut self.items[idx]);
            return Some(old);
        }
        None
    }

    /// Moves the item at `pos` to position `to`, overrides item at `to` in the process,
    /// and leaves the `T::Default()` in `pos`.
    /// # Panics
    /// * if positions `pos` or `to` are out of bounds
    pub fn move_to<P: Into<Position>>(&mut self, pos: P, to: P) {
        let pos = pos.into();
        let to = to.into();

        if !self.is_bounds(pos) && !self.is_bounds(to) {
            panic!("Out of bounds");
        }

        let idx_to = self.translate(to);
        self.items[idx_to] = self.replace_default(pos).unwrap();
    }
}

use crate::{Grid2D, GridIterExt, GridNeighborExt, GridPatternExt, GridDefaultExt};

impl<T> Grid2D for Grid<T> {
    type Item = T;

    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get(&self, x: usize, y: usize) -> Option<&Self::Item> {
        if self.in_bounds(x, y) {
            let idx = self.translate_coords(x, y);
            Some(&self.items[idx])
        } else {
            None
        }
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Item> {
        if self.in_bounds(x, y) {
            let idx = self.translate_coords(x, y);
            Some(&mut self.items[idx])
        } else {
            None
        }
    }

    fn set(&mut self, x: usize, y: usize, item: Self::Item) -> bool {
        if self.in_bounds(x, y) {
            let idx = self.translate_coords(x, y);
            self.items[idx] = item;
            true
        } else {
            false
        }
    }

    fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }
}

impl<T> GridIterExt for Grid<T> {
    type Iter<'a> = GridIter<'a, T> where Self: 'a;
    type IterMut<'a> = GridIterMut<'a, T> where Self: 'a;
    type RowIter<'a> = RowIter<'a, T> where Self: 'a;
    type RowIterMut<'a> = RowIterMut<'a, T> where Self: 'a;
    type ColumnIter<'a> = ColumnIter<'a, T> where Self: 'a;
    type ColumnIterMut<'a> = ColumnIterMut<'a, T> where Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        GridIter {
            grid_iter: self.items.iter(),
            width: self.width,
        }
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        GridIterMut {
            grid_iter: self.items.iter_mut(),
            width: self.width,
        }
    }

    fn row(&self, y: usize) -> Self::RowIter<'_> {
        assert!(self.in_bounds(0, y));
        let start_idx = y * self.width;
        let end_idx = start_idx + self.width;

        RowIter {
            row_iter: self.items[start_idx..end_idx].iter(),
            idx: y,
        }
    }

    fn row_mut(&mut self, y: usize) -> Self::RowIterMut<'_> {
        assert!(self.in_bounds(0, y));
        let start_idx = y * self.width;
        let end_idx = start_idx + self.width;

        RowIterMut {
            row_iter: self.items[start_idx..end_idx].iter_mut(),
            idx: y,
        }
    }

    fn column(&self, x: usize) -> Self::ColumnIter<'_> {
        assert!(self.in_bounds(x, 0));
        ColumnIter {
            row_idx: 0,
            col_idx: x,
            grid: self,
        }
    }

    fn column_mut(&mut self, x: usize) -> Self::ColumnIterMut<'_> {
        assert!(self.in_bounds(x, 0));
        let width = self.width;
        let iter = self.iter_mut().skip(x).step_by(width);
        ColumnIterMut { iter, col_idx: x }
    }
}

impl<T> GridNeighborExt for Grid<T> {
    type NeighborIter<'a> = NeighborIter<'a, T> where Self: 'a;
    type NeighborIterMut<'a> = std::iter::Empty<&'a mut T> where Self: 'a;  // TODO: Implement mutable neighbor iterator

    fn neighbors(&self, x: usize, y: usize) -> Self::NeighborIter<'_> {
        assert!(self.in_bounds(x, y));
        let pos: Position = Position::new(x, y);
        NeighborIter {
            positions: self.get_neighbor_positions(pos),
            grid: self,
            idx: 0,
        }
    }

    fn neighbors_mut(&mut self, _x: usize, _y: usize) -> Self::NeighborIterMut<'_> {
        // TODO: Implement mutable neighbor iterator
        std::iter::empty()
    }
}

impl<T> GridPatternExt for Grid<T> {
    type PatternIter<'a> = PatternIter<'a, T> where Self: 'a;

    fn pattern<P: Pattern + 'static>(&self, x: usize, y: usize, pattern: P) -> Self::PatternIter<'_> {
        PatternIter {
            grid: self,
            origin_position: (x, y).into(),
            prev_position: (x, y).into(),
            pattern: Box::new(pattern),
            repeat_count: 0,
        }
    }
}

impl<T: Default> GridDefaultExt for Grid<T> {
        fn replace_default(&mut self, x: usize, y: usize) -> Option<Self::Item> {
        if self.in_bounds(x, y) {
            let idx = self.translate_coords(x, y);
            let old = mem::take(&mut self.items[idx]);
            Some(old)
        } else {
            None
        }
    }

    fn move_to(&mut self, from_x: usize, from_y: usize, to_x: usize, to_y: usize) -> bool {
        if !self.in_bounds(from_x, from_y) || !self.in_bounds(to_x, to_y) {
            return false;
        }


        let to_idx = self.translate_coords(to_x, to_y);
        let from_pos = Position::new(from_x, from_y);
        self.items[to_idx] = self.replace_default(from_pos).unwrap();
        true
    }
}

impl<T> Grid<T> {


    /// Constructs a new Grid with items in Vector `v`
    /// # Panics
    /// * if width or height is zero
    /// * if `v` length is not equal width times height
    pub fn from(v: Vec<T>, width: usize, height: usize) -> Self {
        if width == 0 || height == 0 {
            panic!("width and height must be positive");
        }
        if v.len() != (width * height) {
            panic!("v length does not equal width * height");
        }
        Self {
            items: v,
            width,
            height,
        }
    }

    #[inline]
    fn translate_coords(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    fn translate<P: Into<Position>>(&self, pos: P) -> usize {
        let pos = pos.into();
        self.translate_coords(pos.x, pos.y)
    }

    /// Checks if position `pos` is in bounds of the grid.
    #[inline]
    pub fn is_bounds<P: Into<Position>>(&self, pos: P) -> bool {
        let pos = pos.into();
        pos.x < self.width && pos.y < self.height
    }

    /// Returns the width and height of the grid.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Returns the full length of the grid
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns a reference to an element at position `pos`
    /// or `None`, if `pos` is out of bounds.
    pub fn get<P: Into<Position>>(&self, pos: P) -> Option<&T> {
        let pos = pos.into();
        if self.is_bounds(pos) {
            let idx = self.translate(pos);
            return Some(&self.items[idx]);
        }
        None
    }

    /// Returns a mutable reference to an element at position `pos`
    /// or None, if `pos` is out of bounds.
    pub fn get_mut<P: Into<Position>>(&mut self, pos: P) -> Option<&mut T> {
        let pos = pos.into();
        if self.is_bounds(pos) {
            let idx = self.translate(pos);
            return Some(&mut self.items[idx]);
        }
        None
    }

    /// Returns a reference to an element at position `pos` without bound checks.
    /// # Safety
    /// Does not do any bound checks.  
    /// `pos` does not have to be in bounds as long pos.x*pos.y < grid.len()  
    /// for example on a grid size 3,3: `get_unchecked(8,0)` will return the last element
    /// # Panics
    /// * if pos.x times pos.y  is greater than grid length.
    pub fn get_unchecked<P: Into<Position>>(&self, pos: P) -> &T {
        let idx = self.translate(pos);
        &self.items[idx]
    }

    /// Returns a reference to an element at position `pos` without bound checks.  
    /// # Safety
    /// Does not do any bound checks.  
    /// `pos` does not have to be in bounds as long pos.x*pos.y < grid.len()  
    /// for example on a grid size 3,3: `get_unchecked(8,0)` will return the last element
    /// # Panics
    /// * if pos.x times pos.y  is greater than grid length.
    pub fn get_mut_unchecked<P: Into<Position>>(&mut self, pos: P) -> &mut T {
        let idx = self.translate(pos);
        &mut self.items[idx]
    }

    /// Sets the value at position `pos`.
    /// Returns None if `pos` is out of bounds,
    /// or () otherwise.
    pub fn set<P: Into<Position>>(&mut self, pos: P, value: T) -> Option<()> {
        let pos = pos.into();
        if self.is_bounds(pos) {
            let idx = self.translate(pos);
            self.items[idx] = value;
        }
        None
    }

    /// Sets the value at position `pos`, without bound checks.
    /// # Safety
    /// Does not do any bound checks.  
    /// `pos` does not have to be in bounds as long pos.x*pos.y < grid.len()  
    /// for example on a grid size 3,3: `get_unchecked(8,0)` will return the last element
    /// # Panics
    /// * if pos.x times pos.y  is greater than grid length.
    pub fn set_unchecked<P: Into<Position>>(&mut self, pos: P, value: T) {
        let idx = self.translate(pos);
        self.items[idx] = value;
    }

    /// Replace the value at position `pos` and returns the old value,
    /// or `None` if `pos` is out of bounds.
    pub fn replace<P: Into<Position>>(&mut self, pos: P, value: T) -> Option<T> {
        let pos = pos.into();
        if self.is_bounds(pos) {
            let idx = self.translate(pos);
            let old = mem::replace(&mut self.items[idx], value);
            return Some(old);
        }
        None
    }

    /// Swap the values of positions `pos_a` and `pos_b`.
    /// # Panics
    /// * if position `pos` is out of bounds.
    pub fn swap<P: Into<Position>>(&mut self, pos_a: P, pos_b: P) {
        let pos_a = pos_a.into();
        let pos_b = pos_b.into();
        if !self.is_bounds(pos_a) && !self.is_bounds(pos_b) {
            panic!("Out of bounds");
        }

        let idx_a = self.translate(pos_a);
        let idx_b = self.translate(pos_b);
        self.items.swap(idx_a, idx_b);
    }

    /// Move the value of position `pos` to position `to` and leaves `value` in it's place.
    /// # Panics
    /// * if position `pos` is out of bounds
    pub fn move_and_leave<P: Into<Position>>(&mut self, pos: P, to: P, value: T) {
        let pos = pos.into();
        let to = to.into();
        if !self.is_bounds(pos) && !self.is_bounds(to) {
            panic!("Out of bound");
        }

        let idx_to = self.translate(to);
        self.items[idx_to] = self.replace(pos, value).unwrap();
    }

    /// Creates an iterator which yields all positions of grid.
    pub fn positions(&self) -> PositionsIter {
        PositionsIter {
            len: self.items.len(),
            width: self.width,
            idx: 0,
        }
    }

    /// Creates an iterator which yields references of every element in grid.
    pub fn iter(&self) -> GridIter<'_, T> {
        GridIter {
            grid_iter: self.items.iter(),
            width: self.width,
        }
    }

    /// Creates an iterator which yields mutable references of every element in grid.
    pub fn iter_mut(&mut self) -> GridIterMut<'_, T> {
        GridIterMut {
            grid_iter: self.items.iter_mut(),
            width: self.width,
        }
    }

    /// Creates an iterator which yields references of every element in row `y`.
    /// # Panics
    /// * if the row is out of bounds.
    pub fn row(&self, y: usize) -> RowIter<'_, T> {
        assert!(self.in_bounds(0, y));
        let start_idx = y * self.width;
        let end_idx = start_idx + self.width;

        RowIter {
            row_iter: self.items[start_idx..end_idx].iter(),
            idx: y,
        }
    }

    /// Creates an iterator which yields mutable references of every element in row `y`.
    /// # Panics
    /// * if the row is out of bounds.
    pub fn row_mut(&mut self, y: usize) -> RowIterMut<'_, T> {
        assert!(self.in_bounds(0, y));
        let start_idx = y * self.width;
        let end_idx = start_idx + self.width;

        RowIterMut {
            row_iter: self.items[start_idx..end_idx].iter_mut(),
            idx: y,
        }
    }

    /// Creates an iterator which yields references of every element in column `x`.
    /// # Panics
    /// * if the column is out of bounds.
    pub fn column(&self, x: usize) -> ColumnIter<'_, T> {
        assert!(self.in_bounds(x, 0));
        ColumnIter {
            row_idx: 0,
            col_idx: x,
            grid: self,
        }
    }

    /// Creates an iterator which yields mutable references of every element in column `x`.
    /// # Panics
    /// * if the column is out of bounds.
    pub fn column_mut(&mut self, x: usize) -> ColumnIterMut<'_, T> {
        assert!(self.in_bounds(x, 0));
        let width = self.width;
        let iter = self.iter_mut().skip(x).step_by(width);
        ColumnIterMut { iter, col_idx: x }
    }

    // Returns every valid neighbor position of x,y
    fn get_neighbor_positions<P: Into<Position>>(&self, pos: P) -> Vec<Position> {
        let Position { x, y } = pos.into();
        let neighbor_position: [(N, N); 8] = [
            (N::N(1), N::N(1)),
            (N::P(0), N::N(1)),
            (N::P(1), N::N(1)),
            (N::N(1), N::P(0)),
            (N::P(1), N::P(0)),
            (N::N(1), N::P(1)),
            (N::P(0), N::P(1)),
            (N::P(1), N::P(1)),
        ];

        let valid_positions: Vec<Position> = neighbor_position
            .iter()
            .filter_map(|(nx, ny)| {
                let x = nx.checked_add_sub(x)?;
                let y = ny.checked_add_sub(y)?;

                if self.get((x, y)).is_some() {
                    return Some((x, y).into());
                }
                None
            })
            .collect();

        valid_positions
    }

    /// Creates an iterator which yields references of every neighbor element of position `pos`.
    /// # Panics
    /// * if x or y is out of bounds.
    pub fn neighbors<P: Into<Position>>(&self, pos: P) -> NeighborIter<'_, T> {
        let pos = pos.into();
        assert!(self.is_bounds(pos));
        NeighborIter {
            positions: self.get_neighbor_positions(pos),
            grid: self,
            idx: 0,
        }
    }

    /// Creates an iterator which yields references of every element of pattern starting at position `pos`.  
    /// See [Pattern] more details.
    pub fn pattern<P, Pat>(&self, pos: P, pattern: Pat) -> PatternIter<'_, T>
    where
        P: Into<Position>,
        Pat: Pattern + 'static,
    {
        let pos = pos.into();
        PatternIter {
            grid: self,
            origin_position: pos,
            prev_position: pos,
            pattern: Box::new(pattern),
            repeat_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_grid() {
        let grid = Grid::new(3, 5, 0u8);
        assert_eq!(
            grid,
            Grid {
                width: 3,
                height: 5,
                items: vec![0u8; 3 * 5]
            }
        );
    }

    #[test]
    fn get_cell_in_grid() {
        let grid = Grid {
            width: 3,
            height: 3,
            items: vec![1, 1, 1, 1, 2, 1, 1, 1, 1],
        };
        let cell = grid.get((1, 1));
        assert_eq!(cell, Some(&2));
    }

    #[test]
    fn get_mut_cell_in_grid() {
        let mut grid = Grid {
            width: 3,
            height: 3,
            items: vec![1, 1, 1, 1, 2, 1, 1, 1, 1],
        };
        let mut_cell = grid.get_mut((1, 1));
        assert_eq!(mut_cell, Some(&mut 2));
    }

    #[test]
    fn get_unchecked_cell_in_grid() {
        let grid = Grid {
            width: 3,
            height: 3,
            items: vec![1, 1, 1, 1, 2, 1, 1, 1, 1],
        };
        let cell = grid.get_unchecked((1, 1));
        assert_eq!(cell, &2);
    }

    #[test]
    #[should_panic]
    fn get_unchecked_panic_cell_in_grid() {
        let grid = Grid {
            width: 3,
            height: 3,
            items: vec![1, 1, 1, 1, 2, 1, 1, 1, 1],
        };
        let _cell = grid.get_unchecked((3, 2));
    }

    #[test]
    fn set_cell_in_grid() {
        let mut grid = Grid::new(3, 5, 1u8);
        grid.set((2, 2), 2u8);
        let cell = grid.get((2, 2));
        assert_eq!(cell, Some(&2));
    }

    #[test]
    fn set_unchecked_cell_in_grid() {
        let mut grid = Grid::new(3, 5, 1u8);
        grid.set_unchecked((2, 2), 2u8);
        let cell = grid.get((2, 2));
        assert_eq!(cell, Some(&2));
    }

    #[test]
    #[should_panic]
    fn set_unchecked_panic_cell_in_grid() {
        let mut grid = Grid::new(3, 3, 1u8);
        grid.set_unchecked((2, 3), 2u8);
    }

    #[test]
    fn replace_cell_in_grid() {
        let mut grid = Grid::new(2, 2, 1u8);
        let value = grid.replace((1, 1), 2u8);
        assert_eq!(value, Some(1));
        assert_eq!(grid.items, vec![1, 1, 1, 2]);
    }

    #[test]
    fn replace_default() {
        let mut grid = Grid::new(2, 2, 1u8);
        grid.replace_default((1, 1));
        assert_eq!(grid.get((1, 1)), Some(&0));
    }

    #[test]
    fn swap() {
        let mut grid = Grid {
            items: (0..6).collect(),
            width: 2,
            height: 3,
        };

        grid.swap((1, 2), (0, 1));
        assert_eq!(grid.get((1, 2)), Some(&2));
        assert_eq!(grid.get((0, 1)), Some(&5));
    }

    #[test]
    fn move_to() {
        let mut grid = Grid {
            items: (0..4).collect(),
            width: 2,
            height: 2,
        };

        grid.move_to((1, 1), (0, 1));
        assert_eq!(grid.get((1, 1)), Some(&0));
        assert_eq!(grid.get((0, 1)), Some(&3));
    }

    #[test]
    fn move_and_leave() {
        let mut grid = Grid {
            items: (0..4).collect(),
            width: 2,
            height: 2,
        };

        grid.move_and_leave((1, 0), (0, 0), 10);
        assert_eq!(grid.get((1, 0)), Some(&10));
        assert_eq!(grid.get((0, 0)), Some(&1));
    }
}
