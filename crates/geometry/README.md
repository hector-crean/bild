# Geometry - Extended Bevy Geometry Representations

A Rust crate that extends Bevy's geometry system with additional primitives and a **granular material system** where each geometry representation is a simple Asset type with its own specific material trait and plugin system.

## Features

### 🎯 Granular Material System

Unlike Bevy's current `Material` trait (which is really `MeshMaterial`), this crate provides a granular material system where **each geometry representation has its own material trait**:

- **`SDFMaterial` trait** - For Signed Distance Fields rendered with raymarching
- **`PointCloudMaterial` trait** - For point clouds rendered as points or instanced geometry  
- **`VoxelMaterial` trait** - For voxel grids rendered with marching cubes (coming soon)
- **`HeightFieldMaterial` trait** - For height fields rendered as displaced geometry (coming soon)
- **And more...**

This approach is more type-safe and allows each representation to have material properties specifically tailored to its rendering needs.

### 🔧 Key Components

#### Simple Asset Types
Each geometry representation is just a simple Asset:

```rust
#[derive(Asset, Clone, Debug, TypePath)]
pub struct SignedDistanceField {
    pub function: SDFFunction,
    pub bounds: BoundingBox,
    pub max_distance: f32,
    pub epsilon: f32,
    pub max_steps: u32,
}

#[derive(Asset, Clone, Debug, TypePath)]  
pub struct PointCloud {
    pub points: Vec<Point3D>,
    pub bounds: Option<BoundingBox>,
}
```

#### `BaseMaterial` Trait
Provides common material functionality that all specific material traits extend:

```rust
pub trait BaseMaterial: Asset + AsBindGroup + Clone + Sized {
    fn vertex_shader() -> ShaderRef { ShaderRef::Default }
    fn fragment_shader() -> ShaderRef { ShaderRef::Default }
    fn compute_shader() -> ShaderRef { ShaderRef::Default } // For compute-based representations
    
    fn alpha_mode(&self) -> AlphaMode { AlphaMode::Opaque }
    fn opaque_render_method(&self) -> OpaqueRendererMethod { OpaqueRendererMethod::Forward }
    // ... other methods similar to Bevy's Material trait
}
```

#### Specific Material Traits
Each representation type has its own material trait:

```rust
// SDF-specific material trait
pub trait SDFMaterial: BaseMaterial {
    fn uses_soft_shadows(&self) -> bool { false }
    fn uses_ambient_occlusion(&self) -> bool { false }
    fn raymarching_params(&self) -> RaymarchingParams { /* ... */ }
    // SDF-specific pipeline specialization
}

// Point cloud-specific material trait  
pub trait PointCloudMaterial: BaseMaterial {
    fn uses_instancing(&self) -> bool { false }
    fn uses_distance_attenuation(&self) -> bool { false }
    fn point_rendering_params(&self) -> PointRenderingParams { /* ... */ }
    // Point cloud-specific pipeline specialization
}
```

#### Representation-Specific Plugins
Each representation has its own plugin system:

```rust
app.add_plugins(SDFPlugin); // Uses SDFMaterialPlugin<StandardSDFMaterial>
app.add_plugins(PointCloudPlugin); // Uses PointCloudMaterialPlugin<StandardPointCloudMaterial>

// Or use custom materials:
app.add_plugins(SDFMaterialPlugin::<MyCustomSDFMaterial>::default());
```

## Usage Examples

### Signed Distance Fields

```rust
use bevy::prelude::*;
use geometry::prelude::*;
use geometry::representation::sdf::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SDFPlugin) // Adds SDF rendering with StandardSDFMaterial
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut sdf_assets: ResMut<Assets<SignedDistanceField>>,
    mut sdf_materials: ResMut<Assets<StandardSDFMaterial>>,
) {
    // Create an SDF sphere - just a simple Asset
    let sphere_sdf = sdf_assets.add(SignedDistanceField::sphere(1.0));
    
    // Create an SDF material with raymarching-specific properties
    let material = sdf_materials.add(StandardSDFMaterial {
        color: LinearRgba::RED,
        metallic: 0.1,
        roughness: 0.3,
        ao_intensity: 0.2,      // SDF-specific: ambient occlusion
        soft_shadows: 1,        // SDF-specific: soft shadow toggle
        shadow_softness: 8.0,   // SDF-specific: shadow softness factor
        ..Default::default()
    });
    
    // Spawn the SDF entity
    commands.spawn(StandardSDFMaterialBundle {
        representation_material: RepresentationMaterial3d {
            representation: sphere_sdf,
            material,
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}
```

### Point Clouds

```rust
use geometry::representation::pointcloud::*;

fn setup(
    mut commands: Commands,
    mut pointcloud_assets: ResMut<Assets<PointCloud>>,
    mut pointcloud_materials: ResMut<Assets<StandardPointCloudMaterial>>,
) {
    // Create a random point cloud - just a simple Asset
    let cloud = pointcloud_assets.add(PointCloud::random_cloud(
        5000,
        BoundingBox {
            min: Vec3::new(-2.0, -2.0, -2.0),
            max: Vec3::new(2.0, 2.0, 2.0),
        }
    ));
    
    // Create a point cloud material with point-specific properties
    let material = pointcloud_materials.add(StandardPointCloudMaterial {
        size_multiplier: 2.0,        // Point cloud-specific: size scaling
        circular_points: 1,          // Point cloud-specific: circular vs square points
        distance_attenuation: 1,     // Point cloud-specific: size based on distance
        min_size: 1.0,              // Point cloud-specific: minimum pixel size
        max_size: 20.0,             // Point cloud-specific: maximum pixel size
        ..Default::default()
    });
    
    // Spawn the point cloud entity
    commands.spawn(StandardPointCloudMaterialBundle {
        representation_material: RepresentationMaterial3d {
            representation: cloud,
            material,
        },
        ..Default::default()
    });
}
```

### Custom Materials

You can easily create custom materials for each representation type:

```rust
// Custom SDF material
#[derive(Asset, AsBindGroup, Clone, TypePath)]
pub struct MyCustomSDFMaterial {
    #[uniform(0)]
    pub custom_property: f32,
    // ... other properties
}

impl BaseMaterial for MyCustomSDFMaterial {
    fn fragment_shader() -> ShaderRef {
        "my_custom_sdf_shader.wgsl".into()
    }
}

impl SDFMaterial for MyCustomSDFMaterial {
    fn uses_soft_shadows(&self) -> bool {
        true // Always use soft shadows for this material
    }
    
    fn raymarching_params(&self) -> RaymarchingParams {
        RaymarchingParams {
            max_steps: 200, // Higher quality raymarching
            epsilon: 0.0001,
            step_scale: 0.8,
        }
    }
}

// Use it with the plugin system
app.add_plugins(SDFMaterialPlugin::<MyCustomSDFMaterial>::default());
```

## Architecture Benefits

### 🚀 **Simplicity**
- No complex trait hierarchies - just simple Asset types
- Each representation is a straightforward struct
- Material traits provide only representation-specific functionality

### 🚀 **Type Safety**
Each material is tied to its specific representation type at compile time:
- `SDFMaterial` can only be used with `SignedDistanceField`
- `PointCloudMaterial` can only be used with `PointCloud`
- No runtime errors from mismatched materials and representations

### 🎨 **Representation-Specific Properties**
Each material type has properties tailored to its rendering approach:
- **SDF Materials**: Raymarching steps, epsilon, ambient occlusion intensity, soft shadow parameters
- **Point Cloud Materials**: Point size scaling, distance attenuation, circular vs square points
- **Future Voxel Materials**: Marching cubes parameters, level-of-detail settings

### 🔧 **Familiar Plugin System**
Uses the same patterns as Bevy's `MaterialPlugin`:
- `SDFPlugin` for standard SDF materials
- `PointCloudPlugin` for standard point cloud materials
- `SDFMaterialPlugin<M>` for custom SDF materials
- `PointCloudMaterialPlugin<M>` for custom point cloud materials

### 🔄 **Easy Extension**
Adding new representation types follows a clear pattern:

```rust
// 1. Define your representation as a simple Asset
#[derive(Asset, Clone, Debug, TypePath)]
pub struct MyRepresentation {
    // ... your data
}

impl MyRepresentation {
    pub fn to_render_data(&self) -> MyRenderData { /* ... */ }
    pub fn supports_batching(&self) -> bool { /* ... */ }
}

// 2. Define your material trait
pub trait MyMaterial: BaseMaterial {
    fn my_specific_method(&self) -> MyParams;
}

// 3. Create a standard implementation
#[derive(Asset, AsBindGroup, Clone, TypePath)]
pub struct StandardMyMaterial { /* ... */ }

impl BaseMaterial for StandardMyMaterial { /* ... */ }
impl MyMaterial for StandardMyMaterial { /* ... */ }

// 4. Create plugin infrastructure
pub struct MyMaterialPlugin<M: MyMaterial> { /* ... */ }
pub struct MyPlugin; // Uses StandardMyMaterial
```

## Comparison with Bevy's Current System

| Bevy's System | This Granular System |
|---------------|-------------------|
| `Material` trait | `BaseMaterial` + specific traits (`SDFMaterial`, etc.) |
| `MaterialPlugin<M>` | `SDFMaterialPlugin<M>`, `PointCloudMaterialPlugin<M>`, etc. |
| `MaterialMeshBundle` | `StandardSDFMaterialBundle`, `StandardPointCloudMaterialBundle`, etc. |
| Works only with `Mesh` | Works with **any** Asset type |
| One-size-fits-all material properties | **Representation-specific** material properties |
| Vertex/fragment shaders only | Vertex/fragment/**compute** shaders |
| Complex trait hierarchies | **Simple Asset types** + specific material traits |

## Current Implementations

- ✅ **Signed Distance Fields** - `SDFMaterial` trait with raymarching-specific properties
- ✅ **Point Clouds** - `PointCloudMaterial` trait with point rendering-specific properties
- 🚧 **Voxel Grids** - `VoxelMaterial` trait (coming soon)
- 🚧 **Height Fields** - `HeightFieldMaterial` trait (coming soon)  
- 🚧 **Metaballs** - `MetaballMaterial` trait (coming soon)

## Examples

Run the example to see the system in action:

```bash
cargo run --example representation_materials
```

This example demonstrates:
- Multiple SDF shapes with `SDFMaterial`-specific properties
- Point clouds with `PointCloudMaterial`-specific properties  
- Real-time material property animation
- Side-by-side comparison of different representation types

## Contributing

This granular approach provides better type safety and more specific material properties than a generic system, while being simpler than complex trait hierarchies. It could potentially be contributed back to Bevy core as an extension to the existing material system.

## License

MIT or Apache-2.0 