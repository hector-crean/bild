use crate::position::Position;

/// Iterator that yields grid positions along a straight line between two points
/// using the integer Bresenham algorithm. Endpoints are included.
#[derive(Clone, Debug)]
pub struct BresenhamIter {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    err: i32,
    finished: bool,
}

impl BresenhamIter {
    /// Construct a new iterator from start to end (inclusive).
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        let x0 = start.0 as i32;
        let y0 = start.1 as i32;
        let x1 = end.0 as i32;
        let y1 = end.1 as i32;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let err = dx + dy;

        Self {
            x0,
            y0,
            x1,
            y1,
            dx,
            dy,
            sx,
            sy,
            err,
            finished: false,
        }
    }
}

impl Iterator for BresenhamIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let current = Position::new(self.x0 as usize, self.y0 as usize);

        if self.x0 == self.x1 && self.y0 == self.y1 {
            self.finished = true;
            return Some(current);
        }

        let e2 = 2 * self.err;
        if e2 >= self.dy {
            self.err += self.dy;
            self.x0 += self.sx;
        }
        if e2 <= self.dx {
            self.err += self.dx;
            self.y0 += self.sy;
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_point() {
        let mut it = BresenhamIter::new((4, 6), (4, 6));
        assert_eq!(it.next(), Some(Position::new(4, 6)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn horizontal() {
        let pts: Vec<_> = BresenhamIter::new((1, 2), (4, 2)).collect();
        assert_eq!(pts, vec![
            Position::new(1, 2),
            Position::new(2, 2),
            Position::new(3, 2),
            Position::new(4, 2),
        ]);
    }

    #[test]
    fn vertical() {
        let pts: Vec<_> = BresenhamIter::new((3, 1), (3, 4)).collect();
        assert_eq!(pts, vec![
            Position::new(3, 1),
            Position::new(3, 2),
            Position::new(3, 3),
            Position::new(3, 4),
        ]);
    }

    #[test]
    fn diagonal() {
        let pts: Vec<_> = BresenhamIter::new((1, 1), (4, 4)).collect();
        assert_eq!(pts, vec![
            Position::new(1, 1),
            Position::new(2, 2),
            Position::new(3, 3),
            Position::new(4, 4),
        ]);
    }
}


