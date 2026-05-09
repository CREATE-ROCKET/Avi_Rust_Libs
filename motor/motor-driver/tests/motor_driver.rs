use core::cell::RefCell;
use core::convert::Infallible;
use std::rc::Rc;

use embedded_hal::pwm::{ErrorType, SetDutyCycle};
use motor_driver::{DriverCommand, DutyError, MotorDriver, SignedDutyPermille, Tb67h450};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pin {
    A,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    FullyOff(Pin),
    FullyOn(Pin),
    Fraction(Pin, u16, u16),
}

#[derive(Debug)]
struct FakePwm {
    pin: Pin,
    max: u16,
    duty: u16,
    log: Rc<RefCell<Vec<Operation>>>,
}

impl FakePwm {
    fn new(pin: Pin, log: Rc<RefCell<Vec<Operation>>>) -> Self {
        Self {
            pin,
            max: 1000,
            duty: 0,
            log,
        }
    }

    fn duty(&self) -> u16 {
        self.duty
    }
}

impl ErrorType for FakePwm {
    type Error = Infallible;
}

impl SetDutyCycle for FakePwm {
    fn max_duty_cycle(&self) -> u16 {
        self.max
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.duty = duty;
        Ok(())
    }

    fn set_duty_cycle_fully_off(&mut self) -> Result<(), Self::Error> {
        self.log.borrow_mut().push(Operation::FullyOff(self.pin));
        self.duty = 0;
        Ok(())
    }

    fn set_duty_cycle_fully_on(&mut self) -> Result<(), Self::Error> {
        self.log.borrow_mut().push(Operation::FullyOn(self.pin));
        self.duty = self.max;
        Ok(())
    }

    fn set_duty_cycle_fraction(&mut self, num: u16, denom: u16) -> Result<(), Self::Error> {
        self.log
            .borrow_mut()
            .push(Operation::Fraction(self.pin, num, denom));
        self.duty = (u32::from(num) * u32::from(self.max) / u32::from(denom)) as u16;
        Ok(())
    }
}

#[test]
fn signed_duty_new_accepts_range_edges_and_zero() {
    assert_eq!(
        SignedDutyPermille::new(-1000),
        Ok(SignedDutyPermille::saturating_new(-1000))
    );
    assert_eq!(SignedDutyPermille::new(0), Ok(SignedDutyPermille::zero()));
    assert_eq!(
        SignedDutyPermille::new(1000),
        Ok(SignedDutyPermille::saturating_new(1000))
    );
}

#[test]
fn signed_duty_new_rejects_values_outside_range() {
    assert_eq!(
        SignedDutyPermille::new(-1001),
        Err(DutyError::TooLarge { value: -1001 })
    );
    assert_eq!(
        SignedDutyPermille::new(1001),
        Err(DutyError::TooLarge { value: 1001 })
    );
}

#[test]
fn signed_duty_saturating_new_clamps_outside_values() {
    assert_eq!(SignedDutyPermille::saturating_new(-1001).raw(), -1000);
    assert_eq!(SignedDutyPermille::saturating_new(1001).raw(), 1000);
}

#[test]
fn signed_duty_from_normalized_saturating_clamps_and_rejects_non_finite_values() {
    assert_eq!(
        SignedDutyPermille::from_normalized_saturating(-2.0).raw(),
        -1000
    );
    assert_eq!(
        SignedDutyPermille::from_normalized_saturating(-1.0).raw(),
        -1000
    );
    assert_eq!(SignedDutyPermille::from_normalized_saturating(0.0).raw(), 0);
    assert_eq!(
        SignedDutyPermille::from_normalized_saturating(1.0).raw(),
        1000
    );
    assert_eq!(
        SignedDutyPermille::from_normalized_saturating(2.0).raw(),
        1000
    );
    assert_eq!(
        SignedDutyPermille::from_normalized_saturating(f32::NAN).raw(),
        0
    );
    assert_eq!(
        SignedDutyPermille::from_normalized_saturating(f32::INFINITY).raw(),
        0
    );
}

#[test]
fn tb67h450_forward_drive_turns_b_off_before_setting_a_duty() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut driver = Tb67h450::new(
        FakePwm::new(Pin::A, Rc::clone(&log)),
        FakePwm::new(Pin::B, Rc::clone(&log)),
    );

    driver
        .drive(DriverCommand::Drive(
            SignedDutyPermille::new(250).expect("valid duty"),
        ))
        .expect("drive succeeds");

    assert_eq!(
        log.borrow().as_slice(),
        [
            Operation::FullyOff(Pin::B),
            Operation::Fraction(Pin::A, 250, 1000),
        ]
    );

    let (pwm_a, pwm_b) = driver.release();
    assert_eq!(pwm_a.duty(), 250);
    assert_eq!(pwm_b.duty(), 0);
}

#[test]
fn tb67h450_reverse_drive_turns_a_off_before_setting_b_duty() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut driver = Tb67h450::new(
        FakePwm::new(Pin::A, Rc::clone(&log)),
        FakePwm::new(Pin::B, Rc::clone(&log)),
    );

    driver
        .drive(DriverCommand::Drive(
            SignedDutyPermille::new(-250).expect("valid duty"),
        ))
        .expect("drive succeeds");

    assert_eq!(
        log.borrow().as_slice(),
        [
            Operation::FullyOff(Pin::A),
            Operation::Fraction(Pin::B, 250, 1000),
        ]
    );

    let (pwm_a, pwm_b) = driver.release();
    assert_eq!(pwm_a.duty(), 0);
    assert_eq!(pwm_b.duty(), 250);
}

#[test]
fn tb67h450_coast_sets_both_outputs_fully_off() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut driver = Tb67h450::new(
        FakePwm::new(Pin::A, Rc::clone(&log)),
        FakePwm::new(Pin::B, Rc::clone(&log)),
    );

    driver.coast().expect("coast succeeds");

    assert_eq!(
        log.borrow().as_slice(),
        [Operation::FullyOff(Pin::A), Operation::FullyOff(Pin::B)]
    );

    let (pwm_a, pwm_b) = driver.release();
    assert_eq!(pwm_a.duty(), 0);
    assert_eq!(pwm_b.duty(), 0);
}

#[test]
fn tb67h450_brake_sets_both_outputs_fully_on() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut driver = Tb67h450::new(
        FakePwm::new(Pin::A, Rc::clone(&log)),
        FakePwm::new(Pin::B, Rc::clone(&log)),
    );

    driver.brake().expect("brake succeeds");

    assert_eq!(
        log.borrow().as_slice(),
        [Operation::FullyOn(Pin::A), Operation::FullyOn(Pin::B)]
    );

    let (pwm_a, pwm_b) = driver.release();
    assert_eq!(pwm_a.duty(), 1000);
    assert_eq!(pwm_b.duty(), 1000);
}

#[test]
fn tb67h450_release_returns_both_pwm_parts() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let driver = Tb67h450::new(
        FakePwm::new(Pin::A, Rc::clone(&log)),
        FakePwm::new(Pin::B, Rc::clone(&log)),
    );

    let (pwm_a, pwm_b) = driver.release();

    assert_eq!(pwm_a.pin, Pin::A);
    assert_eq!(pwm_b.pin, Pin::B);
}
