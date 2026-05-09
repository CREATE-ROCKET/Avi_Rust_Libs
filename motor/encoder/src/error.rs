use core::fmt;

pub type Result<T> = core::result::Result<T, EncoderError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderError {
    InvalidLowLimit,
    InvalidHighLimit,
    InvalidFilterThreshold,
    UnitAlreadyConfigured,
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLowLimit => f.write_str("PCNT low limit must be negative"),
            Self::InvalidHighLimit => f.write_str("PCNT high limit must be positive"),
            Self::InvalidFilterThreshold => {
                f.write_str("PCNT glitch filter threshold must be 1023 cycles or less")
            }
            Self::UnitAlreadyConfigured => f.write_str("PCNT unit is already configured"),
        }
    }
}
