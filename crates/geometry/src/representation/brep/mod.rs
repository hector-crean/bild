// TODO: Implement surface_representations/boundary_representation rendering

use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy::asset::Asset;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderRef, ShaderType, SpecializedMeshPipelineError, SpecializedMeshPipelines, SpecializedRenderPipeline};




#[derive(Asset, Clone, Debug, PartialEq, Eq, Hash, TypePath)]
pub struct BRep {

}


pub trait BRepMaterial: Asset + AsBindGroup + Clone + Sized {
    /// Returns this material's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this material's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    // fn specialize(
    //     pipeline: &BRepPipeline<Self>,
    //     descriptor: &mut RenderPipelineDescriptor,
    //         // layout: &MeshVertexBufferLayoutRef,
    //     key: BRepMaterialPipelineKey<Self>,
    // ) -> Result<(), SpecializedBRepPipelineError> {
    //     Ok(())
    // }
}

pub struct BRepMaterialPipelineKey<M: BRepMaterial> {
    pub material_key: M,
}

pub struct BRepPipeline<M: BRepMaterial> {
    pub material_type: std::marker::PhantomData<M>,
    // BRep-specific pipeline data would go here
}


impl<M: BRepMaterial> BRepPipeline<M> {
    pub fn new() -> Self {
        Self {
            material_type: std::marker::PhantomData,
        }
    }
}



pub struct BRepMaterialPlugin<M: BRepMaterial> {
    pub material_type: std::marker::PhantomData<M>,
}


impl<M: BRepMaterial> Plugin for BRepMaterialPlugin<M> {
     fn build(&self, app: &mut App) {
        // Register the point cloud representation and material as assets
        app.init_asset::<BRep>()
           .init_asset::<M>();
        
        // Add the render app systems
        if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
            // render_app
            //     // .init_resource::<SpecializedRenderPipeline<BRepPipeline<M>>>()
            //     .add_systems(
            //         bevy::render::Render,
            //         (
            //                 Self::queue_brep_material.in_set(bevy::render::RenderSystems::Queue),
            //         )
            //     );
        }
    }
}

impl<M: BRepMaterial> BRepMaterialPlugin<M> {
    pub fn new() -> Self {
        Self {
            material_type: std::marker::PhantomData,
        }
    }
    fn queue_brep_material(
        brep: &BRep,
        material: &M,
        // view_key: super::ViewKey,
        // render_phase:RenderPhase,
    ) {
    }
}


