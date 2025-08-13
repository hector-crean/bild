/// Separate trait for material/physical properties needed for physics simulation
pub trait MaterialProperties {
    fn thermal_conductivity(&self) -> f32;
    fn electrical_resistivity(&self) -> f32;
    fn youngs_modulus(&self) -> f32;
    fn poisson_ratio(&self) -> f32;
    fn density(&self) -> f32;
    fn specific_heat(&self) -> f32;
}

/// Optional trait for blocks that have a specific material/construction type
/// This can be used for legacy compatibility or when you need material-based grouping
pub trait HasMaterialType {
    type MaterialType;
    fn material_type(&self) -> Self::MaterialType;
}
