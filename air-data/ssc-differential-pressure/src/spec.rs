pub trait SscPressureSpec {
    const ADDRESS: u8;
    const P_MIN_PA: f32;
    const P_MAX_PA: f32;
    const OUTPUT_MIN_COUNTS: u16;
    const OUTPUT_MAX_COUNTS: u16;
    const HAS_COMPENSATED_TEMPERATURE: bool;
}

pub struct Sscdrrn005pd2a5;

impl SscPressureSpec for Sscdrrn005pd2a5 {
    const ADDRESS: u8 = 0x28;
    const P_MIN_PA: f32 = -34_473.786;
    const P_MAX_PA: f32 = 34_473.786;
    const OUTPUT_MIN_COUNTS: u16 = 1638;
    const OUTPUT_MAX_COUNTS: u16 = 14745;
    const HAS_COMPENSATED_TEMPERATURE: bool = false;
}
