use crate::spec::EncoderSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountsPerRevolution {
    pub numerator: u64,
    pub denominator: u64,
}

impl CountsPerRevolution {
    pub const fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn as_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

pub const fn motor_counts_per_revolution<S: EncoderSpec>() -> CountsPerRevolution {
    CountsPerRevolution::new(S::MOTOR_COUNTS_PER_REV as u64, 1)
}

pub const fn output_counts_per_revolution<S: EncoderSpec>() -> CountsPerRevolution {
    CountsPerRevolution::new(
        S::MOTOR_COUNTS_PER_REV as u64 * S::GEAR_RATIO_NUMERATOR as u64,
        S::GEAR_RATIO_DENOMINATOR as u64,
    )
}

pub fn counts_to_motor_revolutions<S: EncoderSpec>(counts: i64) -> f64 {
    counts as f64 / S::MOTOR_COUNTS_PER_REV as f64
}

pub fn counts_to_output_revolutions<S: EncoderSpec>(counts: i64) -> f64 {
    let counts_per_rev = output_counts_per_revolution::<S>();
    counts as f64 * counts_per_rev.denominator as f64 / counts_per_rev.numerator as f64
}

pub fn counts_to_output_degrees<S: EncoderSpec>(counts: i64) -> f64 {
    counts_to_output_revolutions::<S>(counts) * 360.0
}

pub fn output_revolutions_to_counts<S: EncoderSpec>(revolutions: f64) -> f64 {
    let counts_per_rev = output_counts_per_revolution::<S>();
    revolutions * counts_per_rev.numerator as f64 / counts_per_rev.denominator as f64
}
