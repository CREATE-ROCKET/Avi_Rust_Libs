#![no_std]

mod tb67h450;
pub use tb67h450::Tb67h450;
pub use tb67h450::Tb67h450Error;

/// Duty比を表す構造体。0.0から1.0の範囲で表される。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Duty(f32);

impl Duty {
    pub fn clamp(value: f32) -> Option<Self> {
        if value.is_nan() {
            None
        } else {
            Some(Self(value.clamp(0.0, 1.0)))
        }
    }

    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum DriverCommand {
    Forward(Duty),
    Reverse(Duty),
    Brake,
    Coast,
}

pub trait MotorDriver {
    type Error;

    fn drive(&mut self, command: DriverCommand) -> Result<(), Self::Error>;

    fn brake(&mut self) -> Result<(), Self::Error> {
        self.drive(DriverCommand::Brake)
    }

    fn coast(&mut self) -> Result<(), Self::Error> {
        self.drive(DriverCommand::Coast)
    }
}

pub trait DrivePin {
    type Error;

    fn low(&mut self) -> Result<(), Self::Error>;
    fn high(&mut self) -> Result<(), Self::Error>;

    fn duty(&mut self, duty: Duty) -> Result<(), Self::Error>;
}
