use bevy::prelude::*;
use bevy::ecs::error::BevyError;
use camera::controller::CameraSettings;

/// Marker for entities that can be dragged in 2D.
#[derive(Component, Default)]
pub struct Draggable2d {
    dragging: bool,
    drag_start_entity_translation: Option<Vec3>,
    drag_start_pointer_world: Option<Vec2>,
}

impl Draggable2d {
    #[inline]
    fn offset(&self) -> Vec2 {
        let start_entity = self.drag_start_entity_translation.unwrap();
        let start_pointer = self.drag_start_pointer_world.unwrap();
        start_entity.truncate() - start_pointer
    }
}

/// Optional axis constraints for 2D dragging.
#[derive(Resource, Clone, Copy)]
pub struct Drag2dSettings {
    pub enabled: bool,
    pub grid_snapping: Option<Vec2>,
    pub axis_constraints: Option<BVec2>,
}

impl Default for Drag2dSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            grid_snapping: None,
            axis_constraints: None,
        }
    }
}

#[derive(Default)]
pub struct DragTransform2dPlugin<T: CameraSettings>(pub T);

impl<T: CameraSettings + Send + Sync + 'static> Plugin for DragTransform2dPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Drag2dSettings>()
            .add_systems(Update, Self::sync_drag_system.run_if(run_criteria::<T>));
    }
}

fn run_criteria<T: CameraSettings>(mode: Res<T>) -> bool {
    // !mode.is_locked()
    let _ = mode; // keep consistent with 3D variant; could gate on lock later
    true
}

impl<T: CameraSettings + Send + Sync + 'static> DragTransform2dPlugin<T> {
    fn sync_drag_system(mut commands: Commands, q_added: Query<Entity, Added<Draggable2d>>) {
        for entity in q_added.iter() {
            commands.entity(entity).observe(Self::on_drag_start.pipe(|result: In<Result>| {
                let _ = result.0.inspect_err(|err| info!("captured error: {err}"));
            }));
            commands.entity(entity).observe(Self::on_drag.pipe(|result: In<Result>| {
                let _ = result.0.inspect_err(|err| info!("captured error: {err}"));
            }));
            commands.entity(entity).observe(Self::on_drag_end.pipe(|result: In<Result>| {
                let _ = result.0.inspect_err(|err| info!("captured error: {err}"));
            }));
        }
    }

    fn on_drag_start(
        drag_start: On<Pointer<DragStart>>,
        mut draggable_query: Query<(Entity, &mut Draggable2d, &Transform)>,
        cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
        mut camera_settings: ResMut<T>,
    ) -> Result<(), BevyError> {
        let (camera, camera_transform) = cameras.single()?;
        let (_entity, mut draggable, transform) = draggable_query.get_mut(drag_start.target())?;

        // compute world position of pointer on the Z plane of the entity (2D: z is constant)
        let screen_pos = drag_start.pointer_location.position;

        let world_pos_2d = camera
            .viewport_to_world_2d(camera_transform, screen_pos)
            .map_err(|_| BevyError::from("Failed to unproject to world 2D"))?;

        draggable.drag_start_entity_translation = Some(transform.translation);
        draggable.drag_start_pointer_world = Some(world_pos_2d);
        draggable.dragging = true;

        camera_settings.lock();

        Ok(())
    }

    fn on_drag(
        drag: On<Pointer<Drag>>,
        mut draggable_query: Query<(Entity, &mut Draggable2d, &mut Transform)>,
        cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
        settings: Res<Drag2dSettings>,
    ) -> Result<(), BevyError> {
        let (camera, camera_transform) = cameras.single()?;
        let (_entity, draggable, mut transform) = draggable_query.get_mut(drag.target())?;

        let screen_pos = drag.pointer_location.position;

        let ptr_world_2d = camera
            .viewport_to_world_2d(camera_transform, screen_pos)
            .map_err(|_| BevyError::from("Failed to unproject to world 2D"))?;

        let mut new_xy = ptr_world_2d + draggable.offset();

        // apply grid snapping if set
        if let Some(grid) = settings.grid_snapping {
            if grid.x > 0.0 {
                new_xy.x = (new_xy.x / grid.x).round() * grid.x;
            }
            if grid.y > 0.0 {
                new_xy.y = (new_xy.y / grid.y).round() * grid.y;
            }
        }

        // axis constraints
        if let Some(axis) = settings.axis_constraints {
            if !axis.x {
                new_xy.x = transform.translation.x;
            }
            if !axis.y {
                new_xy.y = transform.translation.y;
            }
        }

        transform.translation.x = new_xy.x;
        transform.translation.y = new_xy.y;

        Ok(())
    }

    fn on_drag_end(
        drag_end: On<Pointer<DragEnd>>,
        mut draggable_query: Query<(Entity, &mut Draggable2d)>,
        mut camera_settings: ResMut<T>,
    ) -> Result<(), BevyError> {
        let (_entity, mut draggable) = draggable_query.get_mut(drag_end.target())?;
        draggable.dragging = false;
        camera_settings.unlock();
        Ok(())
    }
}

