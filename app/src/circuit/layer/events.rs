use bevy::prelude::*;
use super::types::LayerId;
use super::Layer;

/// Commands to update the layer stack
#[derive(Event, BufferedEvent, Clone, Debug)]
pub enum LayerCommand {
    Add(LayerId),
    Remove(LayerId),
    SetVisible { id: LayerId, visible: bool },
    SetColor { id: LayerId, color: Color },
    SetElevation { id: LayerId, elevation: f32 },
    SetActive(LayerId),
}

/// Events emitted after applying commands to the layer stack
#[derive(Event, BufferedEvent, Clone, Debug)]
pub enum LayerEvent {
    Added { id: LayerId },
    Removed { id: LayerId },
    VisibilityChanged { id: LayerId, visible: bool },
    ColorChanged { id: LayerId, color: Color },
    ElevationChanged { id: LayerId, elevation: f32 },
    ActiveChanged { id: LayerId },
}


