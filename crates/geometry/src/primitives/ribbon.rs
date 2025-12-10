use bevy::math::primitives::{Polyline3d, Primitive3d};
use bevy::prelude::{Measured3d, Vec3};

/// A 3D ribbon primitive defined by a path and cross-section dimensions.
///
/// A ribbon is a 3D shape created by sweeping a rectangular cross-section
/// along a 3D path (polyline). It's useful for representing roads, pipes,
/// or any tubular structure with a rectangular cross-section.
#[derive(Clone, Debug, PartialEq)]
pub struct Ribbon3d {
    /// The path along which the ribbon is swept
    pub path: Polyline3d,
    /// The width of the ribbon (along the local X axis)
    pub width: f32,
    /// The height of the ribbon (along the local Y axis)
    pub height: f32,
}

impl Primitive3d for Ribbon3d {}

impl Default for Ribbon3d {
    fn default() -> Self {
        Self {
            path: Polyline3d::new(vec![]),
            width: 1.0,
            height: 0.1,
        }
    }
}

impl Ribbon3d {
    /// Create a new ribbon from a path and cross-section dimensions
    pub fn new(path: Polyline3d, width: f32, height: f32) -> Self {
        Self {
            path,
            width,
            height,
        }
    }

    /// Create a ribbon from a sequence of vertices
    pub fn from_vertices(vertices: impl Into<Vec<Vec3>>, width: f32, height: f32) -> Self {
        Self {
            path: Polyline3d::new(vertices.into()),
            width,
            height,
        }
    }

    /// Get the cross-sectional area of the ribbon
    #[inline]
    pub fn cross_section_area(&self) -> f32 {
        self.width * self.height
    }

    /// Get the length of the ribbon path
    #[inline]
    pub fn path_length(&self) -> f32 {
        // Calculate the total length of the polyline
        let mut length = 0.0;
        let vertices = &self.path.vertices;
        if vertices.len() < 2 {
            return 0.0;
        }
        for i in 0..vertices.len() - 1 {
            length += (vertices[i + 1] - vertices[i]).length();
        }
        length
    }
}

impl Measured3d for Ribbon3d {
    /// Get the surface area of the ribbon
    #[inline]
    fn area(&self) -> f32 {
        let path_length = self.path_length();
        let perimeter = 2.0 * (self.width + self.height);
        path_length * perimeter
    }

    /// Get the volume of the ribbon
    #[inline]
    fn volume(&self) -> f32 {
        self.path_length() * self.cross_section_area()
    }
} 