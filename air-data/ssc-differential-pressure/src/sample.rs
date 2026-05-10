use crate::status::SscStatus;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DifferentialPressureSample {
    pub differential_pressure_pa: f32,
    pub temperature_c: Option<f32>,
    pub status: SscStatus,
    pub raw_pressure_counts: u16,
    pub raw_temperature_counts: u16,
}
