#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SscStatus {
    Normal,
    CommandMode,
    StaleData,
    DiagnosticFault,
}

impl SscStatus {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::Normal,
            1 => Self::CommandMode,
            2 => Self::StaleData,
            _ => Self::DiagnosticFault,
        }
    }

    pub fn as_bits(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::CommandMode => 1,
            Self::StaleData => 2,
            Self::DiagnosticFault => 3,
        }
    }

    pub fn is_valid(self) -> bool {
        self == Self::Normal
    }
}
