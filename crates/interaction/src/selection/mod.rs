use bevy::{
    ecs::observer::On,
    prelude::*,
};

#[derive(Debug, Resource, Default)]
pub struct SelectionPluginSettings {
    /// Should selection systems run?
    pub is_enabled: bool,
    /// A pointer clicks and nothing is beneath it, should everything be deselected?
    pub click_nothing_deselect_all: bool,
}

/// Marker component indicating this entity is currently selected
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct Selected;

/// Marker component indicating this entity can be selected (clickable)
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct Selectable;

/// Marker component for entities that deselect all selected entities when clicked
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
pub struct Deselecter;

/// Marker struct used to mark pickable entities for which you don't want to trigger a deselection
/// event when picked. This is useful for gizmos or other pickable UI entities.
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct NoDeselect;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(SelectionPluginSettings::default())
        .add_systems(Update, (Self::setup_selectable, Self::setup_deselecters));
    }
}

impl SelectionPlugin {
    fn setup_selectable(mut commands: Commands, query: Query<Entity, Added<Selectable>>) {
        for entity in query.iter() {
            commands
                .entity(entity)
                .observe(Self::handle_click_on_selectable());
        }
    }

    fn setup_deselecters(mut commands: Commands, query: Query<Entity, Added<Deselecter>>) {
        for deselecter_entity in query.iter() {
            commands
                .entity(deselecter_entity)
                .observe(Self::handle_click_on_deselecter());
        }
    }

    fn handle_click_on_selectable() -> impl Fn(On<Pointer<Click>>, Query<&Selected, Without<NoDeselect>>, Commands) {
        move |trigger, selected_query, mut commands| {
            let entity = trigger.target();
            
            // Toggle selection by checking if Selected component exists
            if selected_query.get(entity).is_ok() {
                commands.entity(entity).remove::<Selected>();
            } else {
                commands.entity(entity).insert(Selected);
            }
        }
    }

    fn handle_click_on_deselecter() -> impl Fn(On<Pointer<Click>>, Query<Entity, (With<Selected>, Without<NoDeselect>)>, Commands) {
        move |_, selected_query, mut commands| {
            for entity in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
        }
    }
}
