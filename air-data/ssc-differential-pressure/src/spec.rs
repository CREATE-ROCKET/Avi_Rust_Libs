pub trait SscPressureSpec {
    const ADDRESS: u8;
    const P_MIN_PA: f32;
    const P_MAX_PA: f32;

    /// Transfer function lower endpoint in pressure output counts.
    ///
    /// This may be fractional. For a 14-bit 10% endpoint this is 1638.4,
    /// not the nearest integer received from the sensor.
    const OUTPUT_MIN_COUNTS: f32;

    /// Transfer function upper endpoint in pressure output counts.
    ///
    /// This may be fractional. For a 14-bit 90% endpoint this is 14745.6,
    /// not the nearest integer received from the sensor.
    const OUTPUT_MAX_COUNTS: f32;

    /// Lowest integer pressure count accepted as an in-range sensor output.
    const VALID_OUTPUT_MIN_COUNTS: u16;

    /// Highest integer pressure count accepted as an in-range sensor output.
    const VALID_OUTPUT_MAX_COUNTS: u16;

    const HAS_VALID_TEMPERATURE_OUTPUT: bool;
}

/// SSCDRRN005PD2A5: ±5 psi differential, 5 V supply, I2C address 0x28.
///
/// 圧力出力は温度補償済みだが、補正済み温度出力が有効であるとは扱わない。
pub struct Sscdrrn005pd2a5;

pub type SscDiff5Psi5vI2c = Sscdrrn005pd2a5;

impl SscPressureSpec for Sscdrrn005pd2a5 {
    const ADDRESS: u8 = 0x28;
    const P_MIN_PA: f32 = -34_473.786;
    const P_MAX_PA: f32 = 34_473.786;
    const OUTPUT_MIN_COUNTS: f32 = 1_638.4;
    const OUTPUT_MAX_COUNTS: f32 = 14_745.6;
    const VALID_OUTPUT_MIN_COUNTS: u16 = 1_638;
    const VALID_OUTPUT_MAX_COUNTS: u16 = 14_746;
    const HAS_VALID_TEMPERATURE_OUTPUT: bool = false;
}
