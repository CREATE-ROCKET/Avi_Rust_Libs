#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SscError<I2cError> {
    I2c(I2cError),
    InvalidAddress { address: u8 },
    CommandMode,
    StaleData,
    DiagnosticFault,
    PressureCountsOutOfRange { counts: u16, min: u16, max: u16 },
    InvalidTransferFunction,
    InvalidTemperatureOutput,
}
