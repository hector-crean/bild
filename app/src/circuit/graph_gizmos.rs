

use bevy::{color::palettes, prelude::*};

use super::circuit_graph::{EdgeFrom, EdgeTo};

#[derive(Resource, Clone)]
pub struct GraphGizmosConfig {
	pub line_color: Color,
	pub node_color: Color,
	pub draw_nodes: bool,
}

impl Default for GraphGizmosConfig {
	fn default() -> Self {
		Self {
			line_color: Color::WHITE,
			node_color: palettes::tailwind::AMBER_50.into(),
			draw_nodes: true,
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
		let Ok(ta) = transforms.get(from.0) else { continue; };
		let Ok(tb) = transforms.get(to.0) else { continue; };
		let a = ta.translation();
		let b = tb.translation();
        gizmos.line(a, b, config.line_color);
		if config.draw_nodes {
			gizmos.sphere(a, 0.04, config.node_color);
			gizmos.sphere(b, 0.04, config.node_color);
		}
	}
}


