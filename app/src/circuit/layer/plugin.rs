use std::marker::PhantomData;

use bevy::{
    prelude::*,
    reflect::{GetTypeRegistration, Typed},
};
use grid_2d::Grid2D;

use crate::circuit::layer::LayerBuilder;

use super::{
    Layer,
    events::{LayerCommand, LayerEvent},
    resources::{LayerStack},
    types::OnLayer,
};

pub struct LayerManagerPlugin<
    T: Grid2D + Send + Sync + 'static + FromReflect + Typed + Reflect + GetTypeRegistration + Default,
> {
    phantom: PhantomData<T>,
}

impl<T> Plugin for LayerManagerPlugin<T>
where
    T: Grid2D
        + Send
        + Sync
        + 'static
        + FromReflect
        + Typed
        + Reflect
        + GetTypeRegistration
        + Default,
{
    fn build(&self, app: &mut App) {
        app.register_type::<super::types::LayerId>()
            .register_type::<Layer<T>>()
            .register_type::<OnLayer>()
            .init_resource::<LayerStack<T>>()
            .add_event::<LayerCommand>()
            .add_event::<LayerEvent>()
            .add_systems(
                Update,
                (
                    Self::apply_layer_commands.run_if(on_event::<LayerCommand>),
                    Self::on_layer_added,
                    Self::on_layer_changed,
                    Self::on_layer_removed,
                ),
            );
    }
}

impl<T> LayerManagerPlugin<T>
where
    T: Grid2D
        + Send
        + Sync
        + 'static
        + FromReflect
        + Typed
        + Reflect
        + GetTypeRegistration
        + Default,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn on_layer_added(
        mut layer_stack: ResMut<LayerStack<T>>,
        q_added: Query<(Entity, &OnLayer), Added<OnLayer>>,
    ) {
        for (entity, on_layer) in q_added.iter() {
            let mut layer_ref = layer_stack.add_layer(on_layer.0);

            layer_ref.entities_mut().insert(entity);
        }
    }

    fn on_layer_changed(
        mut index: ResMut<LayerStack<T>>,
        q_changed: Query<(Entity, &OnLayer), Changed<OnLayer>>,
    ) {
        for (entity, on_layer) in q_changed.iter() {
            
        }
    }

    fn on_layer_removed(mut index: ResMut<LayerStack<T>>, mut removed: RemovedComponents<OnLayer>) {
        for entity in removed.read() {
           
        }
    }

    fn apply_layer_commands(
        mut cmd_rdr: EventReader<LayerCommand>,
        mut stack: ResMut<LayerStack<T>>,
        mut evt_wtr: EventWriter<LayerEvent>,
    ) {
        for cmd in cmd_rdr.read() {
            match cmd.clone() {
                LayerCommand::Add(layer_id) => {
                    stack.add_layer(layer_id);
                    evt_wtr.write(LayerEvent::Added { id: layer_id });
                }
                LayerCommand::Remove(id) => {
                    stack.remove_layer(id);
                    // grids.remove(id);
                    evt_wtr.write(LayerEvent::Removed { id });
                }
               _ => {}
            }
        }
    }

}
