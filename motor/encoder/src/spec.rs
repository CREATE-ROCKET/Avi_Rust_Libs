pub trait EncoderSpec {
    const MOTOR_COUNTS_PER_REV: u32;
    const GEAR_RATIO_NUMERATOR: u32;
    const GEAR_RATIO_DENOMINATOR: u32;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pololu99To1_25DHp12V;

impl EncoderSpec for Pololu99To1_25DHp12V {
    const MOTOR_COUNTS_PER_REV: u32 = 48;
    const GEAR_RATIO_NUMERATOR: u32 = 9_878;
    const GEAR_RATIO_DENOMINATOR: u32 = 100;
}
