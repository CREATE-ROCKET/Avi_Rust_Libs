#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DutyError {
    TooLarge { value: i16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedDutyPermille(i16);

impl SignedDutyPermille {
    pub const MIN: i16 = -1000;
    pub const MAX: i16 = 1000;

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn new(value: i16) -> Result<Self, DutyError> {
        if value < Self::MIN || value > Self::MAX {
            Err(DutyError::TooLarge { value })
        } else {
            Ok(Self(value))
        }
    }

    pub const fn saturating_new(value: i16) -> Self {
        let value = if value < Self::MIN {
            Self::MIN
        } else if value > Self::MAX {
            Self::MAX
        } else {
            value
        };

        Self(value)
    }

    pub fn from_normalized_saturating(value: f32) -> Self {
        if !value.is_finite() {
            return Self::zero();
        }

        let scaled = value.clamp(-1.0, 1.0) * f32::from(Self::MAX);
        let rounded = if scaled.is_sign_positive() {
            scaled + 0.5
        } else {
            scaled - 0.5
        };

        Self::saturating_new(rounded as i16)
    }

    pub const fn raw(self) -> i16 {
        self.0
    }

    pub const fn magnitude_permille(self) -> u16 {
        self.0.unsigned_abs()
    }

    pub const fn is_forward(self) -> bool {
        self.0 > 0
    }

    pub const fn is_reverse(self) -> bool {
        self.0 < 0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}
