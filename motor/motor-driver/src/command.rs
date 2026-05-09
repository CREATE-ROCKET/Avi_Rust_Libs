use crate::SignedDutyPermille;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverCommand {
    Drive(SignedDutyPermille),
    Brake,
    Coast,
}
