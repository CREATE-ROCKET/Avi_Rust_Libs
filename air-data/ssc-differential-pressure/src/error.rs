#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SscError<I2cError> {
    I2c(I2cError),
    CommandMode,
    StaleData,
    DiagnosticFault,
    PressureCountsOutOfRange { counts: u16 },
    InvalidTransferFunction,
}
