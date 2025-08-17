//! A 2D Grid Library which utilizes the fun of Iterators.
//! The entry point is the [Grid] struct.   
//! The Grid has iterators for Rows and Columns and also
//! for iterators depending on a [Position].
//! E.g get the neighbor cells of a position with [Grid::neighbors] or
//! cells depending of a pattern from a given position with [Grid::pattern].

pub mod pattern;
pub mod step;
pub mod position;

pub use step::Step;

pub mod vec;








/// Core trait for 2D grid functionality
pub trait Grid2D {
    /// The type of items stored in the grid
    type Item;
    
    /// Returns the dimensions of the grid as (width, height)
    fn dimensions(&self) -> (usize, usize);
    
    /// Returns a reference to the item at the given position, if it exists
    fn get(&self, x: usize, y: usize) -> Option<&Self::Item>;
    
    /// Returns a mutable reference to the item at the given position, if it exists
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Self::Item>;
    
    /// Sets the item at the given position
    /// Returns true if the position was valid and the item was set
    fn set(&mut self, x: usize, y: usize, item: Self::Item) -> bool;
    
    /// Returns whether the given position is within the grid bounds
    fn in_bounds(&self, x: usize, y: usize) -> bool;
}

/// Extension trait for iteration capabilities
pub trait GridIterExt: Grid2D {
    type Iter<'a>: Iterator<Item = &'a Self::Item> where Self: 'a;
    type IterMut<'a>: Iterator<Item = &'a mut Self::Item> where Self: 'a;
    type RowIter<'a>: Iterator<Item = &'a Self::Item> where Self: 'a;
    type RowIterMut<'a>: Iterator<Item = &'a mut Self::Item> where Self: 'a;
    type ColumnIter<'a>: Iterator<Item = &'a Self::Item> where Self: 'a;
    type ColumnIterMut<'a>: Iterator<Item = &'a mut Self::Item> where Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
    fn row(&self, y: usize) -> Self::RowIter<'_>;
    fn row_mut(&mut self, y: usize) -> Self::RowIterMut<'_>;
    fn column(&self, x: usize) -> Self::ColumnIter<'_>;
    fn column_mut(&mut self, x: usize) -> Self::ColumnIterMut<'_>;
}

/// Extension trait for neighbor operations
pub trait GridNeighborExt: Grid2D {
    type NeighborIter<'a>: Iterator<Item = &'a Self::Item> where Self: 'a;
    type NeighborIterMut<'a>: Iterator<Item = &'a mut Self::Item> where Self: 'a;

    fn neighbors(&self, x: usize, y: usize) -> Self::NeighborIter<'_>;
    fn neighbors_mut(&mut self, x: usize, y: usize) -> Self::NeighborIterMut<'_>;
}

use crate::pattern::Pattern;

/// Extension trait for pattern-based operations
pub trait GridPatternExt: Grid2D {
    type PatternIter<'a>: Iterator<Item = &'a Self::Item> where Self: 'a;
    
    fn pattern<P: Pattern + 'static>(&self, x: usize, y: usize, pattern: P) -> Self::PatternIter<'_>;
}

/// Extension trait for default value operations
pub trait GridDefaultExt: Grid2D where Self::Item: Default {
    fn replace_default(&mut self, x: usize, y: usize) -> Option<Self::Item>;
    fn move_to(&mut self, from_x: usize, from_y: usize, to_x: usize, to_y: usize) -> bool;
}

