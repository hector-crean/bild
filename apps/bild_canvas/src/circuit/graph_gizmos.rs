use bevy::{color::palettes, prelude::*};

use super::graph::{EdgeFrom, EdgeTo};

#[derive(Resource, Clone)]
pub struct GraphGizmosConfig {
    pub line_color: Color,
    pub node_color: Color,
    pub draw_nodes: bool,
    pub draw_arrows: bool,
    pub arrow_size: f32,
}

impl Default for GraphGizmosConfig {
    fn default() -> Self {
        Self {
            line_color: Color::WHITE,
            node_color: palettes::tailwind::AMBER_50.into(),
            draw_nodes: true,
            draw_arrows: true,
            arrow_size: 0.15,
        }
    }
}

pub struct GraphGizmosPlugin;

impl Plugin for GraphGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphGizmosConfig>()
            .add_systems(Update, draw_graph_gizmos);
    }
}

fn draw_graph_gizmos(
    mut gizmos: Gizmos,
    config: Res<GraphGizmosConfig>,
    edges: Query<(&EdgeFrom, &EdgeTo)>,
    transforms: Query<&GlobalTransform>,
) {
    for (from, to) in edges.iter() {
        let Ok(ta) = transforms.get(from.0) else {
            continue;
        };
        let Ok(tb) = transforms.get(to.0) else {
            continue;
        };
        let a = ta.translation();
        let b = tb.translation();
        gizmos.line(a, b, config.line_color);

        // Draw directional arrow pointing from source to target
        if config.draw_arrows {
            let direction = (b - a).normalize();
            // Position arrow near the end, but not exactly at the end to avoid overlap with node
            let arrow_start = b - direction * (0.1 + config.arrow_size * 0.5);
            let arrow_end = b - direction * 0.1;
            gizmos.arrow(arrow_start, arrow_end, config.line_color);
        }

        if config.draw_nodes {
            gizmos.sphere(a, 0.04, config.node_color);
            gizmos.sphere(b, 0.04, config.node_color);
        }
    }
}
