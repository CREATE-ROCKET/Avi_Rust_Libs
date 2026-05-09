use embedded_hal::pwm::{ErrorType, SetDutyCycle};

use crate::{DriverCommand, MotorDriver, SignedDutyPermille};

type DriverResult<PwmA, PwmB> =
    Result<(), Tb67h450Error<<PwmA as ErrorType>::Error, <PwmB as ErrorType>::Error>>;

#[derive(Debug)]
pub struct Tb67h450<PwmA, PwmB>
where
    PwmA: SetDutyCycle,
    PwmB: SetDutyCycle,
{
    pwm_a: PwmA,
    pwm_b: PwmB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tb67h450Error<PwmAError, PwmBError> {
    PwmA(PwmAError),
    PwmB(PwmBError),
}

impl<PwmA, PwmB> Tb67h450<PwmA, PwmB>
where
    PwmA: SetDutyCycle,
    PwmB: SetDutyCycle,
{
    pub fn new(pwm_a: PwmA, pwm_b: PwmB) -> Self {
        Self { pwm_a, pwm_b }
    }

    pub fn release(self) -> (PwmA, PwmB) {
        (self.pwm_a, self.pwm_b)
    }

    fn set_a_off(&mut self) -> DriverResult<PwmA, PwmB> {
        self.pwm_a
            .set_duty_cycle_fully_off()
            .map_err(Tb67h450Error::PwmA)
    }

    fn set_b_off(&mut self) -> DriverResult<PwmA, PwmB> {
        self.pwm_b
            .set_duty_cycle_fully_off()
            .map_err(Tb67h450Error::PwmB)
    }

    fn set_a_on(&mut self) -> DriverResult<PwmA, PwmB> {
        self.pwm_a
            .set_duty_cycle_fully_on()
            .map_err(Tb67h450Error::PwmA)
    }

    fn set_b_on(&mut self) -> DriverResult<PwmA, PwmB> {
        self.pwm_b
            .set_duty_cycle_fully_on()
            .map_err(Tb67h450Error::PwmB)
    }

    fn set_a_permille(&mut self, duty: u16) -> DriverResult<PwmA, PwmB> {
        self.pwm_a
            .set_duty_cycle_fraction(duty, SignedDutyPermille::MAX as u16)
            .map_err(Tb67h450Error::PwmA)
    }

    fn set_b_permille(&mut self, duty: u16) -> DriverResult<PwmA, PwmB> {
        self.pwm_b
            .set_duty_cycle_fraction(duty, SignedDutyPermille::MAX as u16)
            .map_err(Tb67h450Error::PwmB)
    }
}

impl<PwmA, PwmB> MotorDriver for Tb67h450<PwmA, PwmB>
where
    PwmA: SetDutyCycle,
    PwmB: SetDutyCycle,
{
    type Error = Tb67h450Error<PwmA::Error, PwmB::Error>;

    fn drive(&mut self, command: DriverCommand) -> Result<(), Self::Error> {
        match command {
            DriverCommand::Drive(duty) if duty.is_forward() => {
                self.set_b_off()?;
                self.set_a_permille(duty.magnitude_permille())
            }
            DriverCommand::Drive(duty) if duty.is_reverse() => {
                self.set_a_off()?;
                self.set_b_permille(duty.magnitude_permille())
            }
            DriverCommand::Drive(_) | DriverCommand::Coast => {
                self.set_a_off()?;
                self.set_b_off()
            }
            DriverCommand::Brake => {
                self.set_a_on()?;
                self.set_b_on()
            }
        }
    }
}
