use bevy::{color::Color, ecs::entity::Entities, platform::collections::HashSet, reflect::Reflect, ecs::entity::Entity};

use crate::circuit::layer::types::LayerId;

use grid_2d::{Grid2D, GridDefaultExt, GridIterExt, GridNeighborExt, GridPatternExt};

pub mod types;
pub mod resources;
pub mod events;
pub mod plugin;
pub mod grid;



/// Metadata for a fabrication layer
#[derive(Clone, Debug, Reflect)]
pub struct Layer<T: Grid2D + Default> {
    pub id: LayerId,
    pub grid: T,
    pub entities: HashSet<Entity>,
    pub color: Color,
    pub elevation: f32,
    pub visible: bool,
}

pub struct LayerBuilder<T: Grid2D + Default> {
    layer: Layer<T>,
}

impl<T: Grid2D + Default> LayerBuilder<T> {
    pub fn new(id: LayerId) -> Self {
        Self {
            layer: Layer {
                id,
                grid: T::default(),
                entities: HashSet::new(),
                color: Color::WHITE,
                elevation: 0.0,
                visible: true,
            }
        }
    }

    pub fn grid(mut self, grid: T) -> Self {
        self.layer.grid = grid;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.layer.color = color;
        self
    }

    pub fn elevation(mut self, elevation: f32) -> Self {
        self.layer.elevation = elevation;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.layer.visible = visible;
        self
    }

    pub fn build(self) -> Layer<T> {
        self.layer
    }
}

impl<T: Grid2D + Default> Layer<T> {
    fn new(id: LayerId) -> Self {
        Self {
            id,
            grid: T::default(),
            color: Color::WHITE,
            entities: HashSet::new(),
            elevation: 0.0,
            visible: true,
        }
    }
}