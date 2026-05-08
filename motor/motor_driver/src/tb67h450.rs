use embedded_hal::pwm::SetDutyCycle;

use crate::MotorDriver;

#[derive(Debug)]
pub struct Tb67h450<PwmA: SetDutyCycle, PwmB: SetDutyCycle> {
    pwm_a: PwmA,
    pwm_b: PwmB,
}

#[derive(Debug)]
pub enum Tb67h450Error<PwmAError, PwmBError> {
    PwmA(PwmAError),
    PwmB(PwmBError),
}

impl<PwmA: SetDutyCycle, PwmB: SetDutyCycle> Tb67h450<PwmA, PwmB> {
    pub fn new(pwm_a: PwmA, pwm_b: PwmB) -> Self {
        Self { pwm_a, pwm_b }
    }

    pub fn drop(self) -> (PwmA, PwmB) {
        (self.pwm_a, self.pwm_b)
    }

    fn set_duty(&self, duty: crate::Duty, max_duty: u16) -> u16 {
        unsafe { ((duty.as_f32() * max_duty as f32) + 0.5).to_int_unchecked() }
    }
}

impl<PwmA, PwmB> MotorDriver for Tb67h450<PwmA, PwmB>
where
    PwmA: SetDutyCycle,
    PwmB: SetDutyCycle,
{
    type Error = Tb67h450Error<PwmA::Error, PwmB::Error>;

    fn drive(&mut self, command: crate::DriverCommand) -> Result<(), Self::Error> {
        match command {
            crate::DriverCommand::Forward(duty) => {
                self.pwm_b
                    .set_duty_cycle_fully_off()
                    .map_err(Tb67h450Error::PwmB)?;
                self.pwm_a
                    .set_duty_cycle(self.set_duty(duty, self.pwm_a.max_duty_cycle()))
                    .map_err(Tb67h450Error::PwmA)?;
                Ok(())
            }
            crate::DriverCommand::Reverse(duty) => {
                self.pwm_a
                    .set_duty_cycle_fully_off()
                    .map_err(Tb67h450Error::PwmA)?;
                self.pwm_b
                    .set_duty_cycle(self.set_duty(duty, self.pwm_b.max_duty_cycle()))
                    .map_err(Tb67h450Error::PwmB)?;
                Ok(())
            }
            crate::DriverCommand::Brake => {
                self.pwm_a
                    .set_duty_cycle_fully_on()
                    .map_err(Tb67h450Error::PwmA)?;
                self.pwm_b
                    .set_duty_cycle_fully_on()
                    .map_err(Tb67h450Error::PwmB)?;
                Ok(())
            }
            crate::DriverCommand::Coast => {
                self.pwm_a
                    .set_duty_cycle_fully_off()
                    .map_err(Tb67h450Error::PwmA)?;
                self.pwm_b
                    .set_duty_cycle_fully_off()
                    .map_err(Tb67h450Error::PwmB)?;
                Ok(())
            }
        }
    }
}
