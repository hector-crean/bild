use bevy::asset::RenderAssetUsages;
use bevy::prelude::{MeshBuilder, Meshable};
use bevy::mesh::Mesh;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::math::primitives::Polyline3d;

// Builder for Polyline3d
pub struct Polyline3dMeshBuilder {
    polyline: Polyline3d,
    // Could add additional configuration options here
}

// Add wrapper types
pub struct PolylineWrapper(pub Polyline3d);

// Implement Meshable for wrapper types instead
impl Meshable for PolylineWrapper {
    type Output = Polyline3dMeshBuilder;
    
    fn mesh(&self) -> Self::Output {
        Polyline3dMeshBuilder {
            polyline: self.0.clone(),
        }
    }
}

impl MeshBuilder for Polyline3dMeshBuilder {
    fn build(&self) -> Mesh {
        // Convert vertices to positions
        let positions: Vec<[f32; 3]> = self.polyline.vertices
            .iter()
            .map(|v| v.to_array())
            .collect();

        Mesh::new(
            PrimitiveTopology::LineStrip,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    }
}

// Update From implementations
impl From<PolylineWrapper> for Mesh {
    fn from(polyline: PolylineWrapper) -> Self {
        polyline.mesh().build()
    }
}