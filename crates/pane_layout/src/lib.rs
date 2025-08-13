//! Resizable, divider-able panes for Bevy.

pub mod components;
mod handlers;
mod pane_drop_area;
pub mod registry;
mod ui;

/// The Bevy Pane Layout system.
/// The intent of this system is to provide a way to create resizable, split-able panes in Bevy.
/// Mimicking the behavior of of Blender's layout system.
///
/// Blender Documentation: <https://docs.blender.org/manual/en/latest/interface/window_system/areas.html>
///
/// Requirements for a valid Pane:
/// - All panes must fit within their bounds, no overflow is allowed.
/// - Panes can not have power over the layout system, their dimensions are controlled by the layout system and should not be modified by anything else.
/// - All panes must have a header, a content area, however a footer is optional.
/// - Panes cannot have min/max sizes, they must be able to be resized to any size.
///   - If a pane can not be sensibly resized, it can overflow under the other panes.
/// - Panes must not interfere with each other, only temporary/absolute positioned elements are allowed to overlap panes.
use bevy::{asset::uuid::Uuid, picking::pointer::{Location, PointerId, PointerInput}, prelude::*, render::{camera::NormalizedRenderTarget, render_resource::Extent3d}};
use styles::Theme;

use crate::{
    registry::PaneRegistryPlugin,
    ui::{spawn_divider, spawn_pane, spawn_resize_handle},
};

/// Crate prelude.
pub mod prelude {
    pub use crate::{
        PaneAreaNode, PaneContentNode, PaneHeaderNode,
        components::*,
        registry::{PaneAppExt, PaneStructure},
        ui::{spawn_pane, spawn_divider, spawn_resize_handle},
        clamp_ratio, Divider, ResizeHandle, RootPaneLayoutNode,
    };
}




pub trait PaneView: Component {
    fn camera_id(&self) -> Entity;
}






/// The Bevy Pane Layout Plugin.
pub struct PaneLayoutPlugin;

impl Plugin for PaneLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PaneRegistryPlugin)
            .init_resource::<DragState>()
            .add_systems(
                Update,
                (cleanup_divider_single_child, apply_size)
                    .chain()
                    .in_set(PaneLayoutSet),
            );
    }
}

fn apply_size(
    mut query: Query<(Entity, &Size, &mut Node), Changed<Size>>,
    divider_query: Query<&Divider>,
    parent_query: Query<&ChildOf>,
) {
    for (entity, size, mut style) in &mut query {
        let parent = parent_query.get(entity).unwrap().parent();
        let Ok(e) = divider_query.get(parent) else {
            style.width = Val::Percent(100.);
            style.height = Val::Percent(100.);
            continue;
        };

        match e {
            Divider::Horizontal => {
                style.width = Val::Percent(size.0 * 100.);
                style.height = Val::Percent(100.);
            }
            Divider::Vertical => {
                style.width = Val::Percent(100.);
                style.height = Val::Percent(size.0 * 100.);
            }
        }
    }
}

#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
    offset: f32,
    min: f32,
    max: f32,
    parent_node_size: f32,
}

/// System Set to set up the Pane Layout.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaneLayoutSet;



pub fn clamp_ratio(r: f32) -> f32 { r.clamp(0.05, 0.95) }


/// Removes a divider from the hierarchy when it has only one child left, replacing itself with that child.
fn cleanup_divider_single_child(
    mut commands: Commands,
    mut query: Query<(Entity, &Children, &ChildOf), (Changed<Children>, With<Divider>)>,
    mut size_query: Query<&mut Size>,
    children_query: Query<&Children>,
    resize_handle_query: Query<(), With<ResizeHandle>>,
) {
    for (entity, children, parent) in &mut query {
        let mut iter = children
            .iter()
            .filter(|child| !resize_handle_query.contains(*child));
        let child = iter.next().unwrap();
        if iter.next().is_some() {
            continue;
        }

        let size = size_query.get(entity).unwrap().0;
        size_query.get_mut(child).unwrap().0 = size;

        // Find the index of this divider among its siblings
        let siblings = children_query.get(parent.parent()).unwrap();
        let index = siblings.iter().position(|s| s == entity).unwrap();

        commands
            .entity(parent.parent())
            .insert_children(index, &[child]);
        commands.entity(entity).despawn();
    }
}

/// A node that divides an area into multiple areas along an axis.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Divider {
    Horizontal,
    Vertical,
}

#[derive(Component)]
pub struct ResizeHandle;

/// The fraction of space this element takes up in the [`Divider`] it's a child of.
#[derive(Component)]
struct Size(f32);

/// Root node to capture all editor UI elements, nothing but the layout system should modify this.
#[derive(Component)]
pub struct RootPaneLayoutNode;

/// Root node for each pane, holds all event nodes for layout and the basic structure for all Panes.
#[derive(Component)]
struct PaneRootNode {
    name: String,
}

/// Node to denote the area of the Pane.
#[derive(Component, Clone, Default)]
pub struct PaneAreaNode;

/// Node to add widgets into the header of a Pane.
#[derive(Component, Clone, Default)]
pub struct PaneHeaderNode;

/// Node to denote the content space of the Pane.
#[derive(Component, Clone, Default)]
pub struct PaneContentNode;



pub fn pointer_id_from_entity(entity: Entity) -> PointerId {
    let bits = entity.to_bits();
    PointerId::Custom(Uuid::from_u64_pair(bits, bits))
}





/// A viewport is considered active while the mouse is hovering over it.
#[derive(Component)]
pub struct Active;

// FIXME: This system makes a lot of assumptions and is therefore rather fragile. Does not handle multiple windows.
/// Sends copies of [`PointerInput`] event actions from the mouse pointer to pointers belonging to the viewport panes.
pub fn render_target_picking_passthrough<View: PaneView>(
    viewports: Query<(Entity, &View)>,
    content: Query<&PaneContentNode>,
    children_query: Query<&Children>,
    node_query: Query<(&ComputedNode, &UiGlobalTransform, &ImageNode), With<Active>>,
    mut pointer_input_reader: EventReader<PointerInput>,
    // Using commands to output PointerInput events to avoid clashing with the EventReader
    mut commands: Commands,
) {
    for event in pointer_input_reader.read() {
        // Ignore the events sent from this system by only copying events that come directly from the mouse.
        if event.pointer_id != PointerId::Mouse {
            continue;
        }
        for (pane_root, _viewport) in &viewports {
            let content_node_id = children_query
                .iter_descendants(pane_root)
                .find(|e| content.contains(*e))
                .unwrap();

            let image_id = children_query.get(content_node_id).unwrap()[0];
            let Ok((computed_node, global_transform, ui_image)) = node_query.get(image_id) else {
                // Inactive viewport
                continue;
            };
            let node_top_left = global_transform.translation - computed_node.size() / 2.;
            let position = event.location.position - node_top_left;
            let target = NormalizedRenderTarget::Image(ui_image.image.clone().into());

            let event_copy = PointerInput {
                action: event.action,
                location: Location { position, target },
                pointer_id: pointer_id_from_entity(pane_root),
            };

            commands.write_event(event_copy);
        }
    }
}





pub fn update_render_target_size<View: PaneView>(
    query: Query<(Entity, &View)>,
    mut camera_query: Query<&Camera>,
    bodies: Query<&PaneContentNode>,
    children_query: Query<&Children>,
    computed_node_query: Query<&ComputedNode, Changed<ComputedNode>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (pane_root, viewport) in &query {
        let Some(pane_body) = children_query
            .iter_descendants(pane_root)
            .find(|e| bodies.contains(*e))
        else {
            continue;
        };

        let Ok(computed_node) = computed_node_query.get(pane_body) else {
            continue;
        };
        // TODO Convert to physical pixels
        let content_node_size = computed_node.size();

        let camera = camera_query.get_mut(viewport.camera_id()).unwrap();

        let image_handle = camera.target.as_image().unwrap();
        let size = Extent3d {
            width: u32::max(1, content_node_size.x as u32),
            height: u32::max(1, content_node_size.y as u32),
            depth_or_array_layers: 1,
        };
        images.get_mut(image_handle).unwrap().resize(size);
    }
}
