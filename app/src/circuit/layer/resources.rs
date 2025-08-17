use std::collections::HashMap;
use bevy::{color::Color, ecs::{entity::Entity, resource::Resource}, platform::collections::HashSet};
use grid_2d::Grid2D;
use crate::circuit::layer::{Layer, LayerBuilder, LayerId};

pub struct BiMap<K, V> where K: std::hash::Hash + Eq + Copy, V: std::hash::Hash + Eq + Copy {
    forward: HashMap<K, V>,
    reverse: HashMap<V, HashSet<K>>,
}

impl<K, V> Default for BiMap<K, V> 
where K: std::hash::Hash + Eq + Copy, 
      V: std::hash::Hash + Eq + Copy {
    fn default() -> Self {
        Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
        }
    }
}

impl<K, V> BiMap<K, V> 
where K: std::hash::Hash + Eq + Copy,
      V: std::hash::Hash + Eq + Copy {
    pub fn insert(&mut self, k: K, v: V) {
        // Remove old mapping if it exists
        if let Some(old_v) = self.forward.get(&k).copied() {
            self.reverse.get_mut(&old_v).map(|set| {
                set.remove(&k);
            });
        }

        // Add new mapping
        self.forward.insert(k, v);
        self.reverse.entry(v)
            .or_insert_with(HashSet::new)
            .insert(k);
    }

    pub fn remove_key(&mut self, k: &K) -> Option<V> {
        self.forward.remove(k).map(|v| {
            self.reverse.get_mut(&v).map(|set| {
                set.remove(k);
            });
            v
        })
    }

    pub fn remove_value(&mut self, v: &V) -> HashSet<K> {
        let keys = self.reverse.remove(v).unwrap_or_default();
        for k in &keys {
            self.forward.remove(k);
        }
        keys
    }

    pub fn get_value(&self, k: &K) -> Option<&V> {
        self.forward.get(k)
    }

    pub fn get_keys(&self, v: &V) -> Option<&HashSet<K>> {
        self.reverse.get(v)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.forward.contains_key(k)
    }

    pub fn contains_value(&self, v: &V) -> bool {
        self.reverse.contains_key(v)
    }
}

#[derive(Resource)]
pub struct LayerStack<T: Grid2D + Default> {
    pub layers: HashMap<LayerId, Layer<T>>,
    entity_layer_map: BiMap<Entity, LayerId>,
}

impl<T: Grid2D + Default> Default for LayerStack<T> {
    fn default() -> Self {
        Self {
            layers: HashMap::new(),
            entity_layer_map: BiMap::default(),
        }
    }
}

pub struct LayerRef<'a, T: Grid2D + Default> {
    layer: &'a mut Layer<T>,
}

impl<'a, T: Grid2D + Default> LayerRef<'a, T> {
    fn new(layer: &'a mut Layer<T>) -> Self {
        Self { layer }
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

    pub fn grid(mut self, grid: T) -> Self {
        self.layer.grid = grid;
        self
    }

    pub fn get(&self) -> &Layer<T> {
        self.layer
    }
    pub fn entities_mut(&mut self) -> &mut HashSet<Entity> {
        &mut self.layer.entities
    }
}

impl<T: Grid2D + Default> LayerStack<T> {
    pub fn add_layer(&mut self, layer_id: LayerId) -> LayerRef<T> {
        let layer = LayerBuilder::<T>::new(layer_id).build();
        self.layers.insert(layer_id, layer);
        LayerRef::new(self.layers.get_mut(&layer_id).unwrap())
    }

    pub fn layer(&self, id: LayerId) -> Option<&Layer<T>> {
        self.layers.get(&id)
    }

    pub fn layer_mut(&mut self, id: LayerId) -> Option<LayerRef<T>> {
        self.layers.get_mut(&id).map(LayerRef::new)
    }

    pub fn remove_layer(&mut self, id: LayerId) -> Option<Layer<T>> {
        self.entity_layer_map.remove_value(&id);
        self.layers.remove(&id)
    }

    pub fn add_entity_to_layer(&mut self, entity: Entity, layer_id: LayerId) -> bool {
        if !self.layers.contains_key(&layer_id) {
            return false;
        }
        self.entity_layer_map.insert(entity, layer_id);
        true
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.entity_layer_map.remove_key(&entity);
    }

    pub fn get_entity_layer(&self, entity: Entity) -> Option<LayerId> {
        self.entity_layer_map.get_value(&entity).copied()
    }

    pub fn get_layer_entities(&self, layer_id: LayerId) -> Option<&HashSet<Entity>> {
        self.entity_layer_map.get_keys(&layer_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer<T>> {
        self.layers.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Layer<T>> {
        self.layers.values_mut()
    }
}