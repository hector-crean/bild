pub struct PolylinePlugin;

pub const SHADER_HANDLE: Handle<Shader> = weak_handle!("b180bfe9-10c8-48fe-b27a-dfa41436d7d0");

impl Plugin for PolylinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            SHADER_HANDLE,
            "shaders/polyline.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            PolylineBasePlugin,
            PolylineRenderPlugin,
            PolylineMaterialPlugin,
        ));
    }
}