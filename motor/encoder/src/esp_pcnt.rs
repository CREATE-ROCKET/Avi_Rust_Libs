use core::{cell::RefCell, marker::PhantomData, sync::atomic::Ordering};

use critical_section::Mutex;
use esp_hal::{
    gpio::{Input, InputConfig, InputPin, Pull},
    pcnt::{Pcnt, channel, unit},
};
use portable_atomic::AtomicI64;

use crate::{
    conversion, error::EncoderError, spec::EncoderSpec, traits::Encoder, traits::EncoderCount,
    traits::QuadratureEncoder, traits::ResettableEncoder,
};

pub const DEFAULT_LOW_LIMIT: i16 = -30_000;
pub const DEFAULT_HIGH_LIMIT: i16 = 30_000;
pub const MAX_GLITCH_FILTER_CYCLES: u16 = 1_023;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcntConfig {
    pub low_limit: i16,
    pub high_limit: i16,
    pub glitch_filter_cycles: Option<u16>,
    pub pull: Pull,
}

impl PcntConfig {
    pub const fn with_limits(mut self, low_limit: i16, high_limit: i16) -> Self {
        self.low_limit = low_limit;
        self.high_limit = high_limit;
        self
    }

    pub const fn with_glitch_filter_cycles(mut self, cycles: u16) -> Self {
        self.glitch_filter_cycles = Some(cycles);
        self
    }

    pub const fn without_glitch_filter(mut self) -> Self {
        self.glitch_filter_cycles = None;
        self
    }

    pub const fn with_pull(mut self, pull: Pull) -> Self {
        self.pull = pull;
        self
    }
}

impl Default for PcntConfig {
    fn default() -> Self {
        Self {
            low_limit: DEFAULT_LOW_LIMIT,
            high_limit: DEFAULT_HIGH_LIMIT,
            glitch_filter_cycles: None,
            pull: Pull::None,
        }
    }
}

pub struct EspPcntEncoder<const UNIT: usize, S> {
    counter: unit::Counter<'static, UNIT>,
    _pin_a: Input<'static>,
    _pin_b: Input<'static>,
    _spec: PhantomData<S>,
}

impl<const UNIT: usize, S> EspPcntEncoder<UNIT, S>
where
    S: EncoderSpec,
{
    pub fn position_counts(&self) -> i64 {
        accumulator_for_unit::<UNIT>().load(Ordering::SeqCst) + self.counter.get() as i64
    }

    pub fn raw_unit_counts(&self) -> i16 {
        self.counter.get()
    }

    pub fn output_revolutions(&self) -> f64 {
        conversion::counts_to_output_revolutions::<S>(self.position_counts())
    }

    pub fn output_degrees(&self) -> f64 {
        conversion::counts_to_output_degrees::<S>(self.position_counts())
    }

    pub fn reset_position(&mut self) {
        accumulator_for_unit::<UNIT>().store(-(self.counter.get() as i64), Ordering::SeqCst);
    }
}

impl<const UNIT: usize, S> Encoder for EspPcntEncoder<UNIT, S>
where
    S: EncoderSpec,
{
    type Error = EncoderError;

    fn count(&self) -> Result<EncoderCount, Self::Error> {
        Ok(EncoderCount::new(self.position_counts()))
    }
}

impl<const UNIT: usize, S> ResettableEncoder for EspPcntEncoder<UNIT, S>
where
    S: EncoderSpec,
{
    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reset_position();
        Ok(())
    }
}

impl<const UNIT: usize, S> QuadratureEncoder for EspPcntEncoder<UNIT, S> where S: EncoderSpec {}

static UNIT0: Mutex<RefCell<Option<unit::Unit<'static, 0>>>> = Mutex::new(RefCell::new(None));
static UNIT1: Mutex<RefCell<Option<unit::Unit<'static, 1>>>> = Mutex::new(RefCell::new(None));
static UNIT2: Mutex<RefCell<Option<unit::Unit<'static, 2>>>> = Mutex::new(RefCell::new(None));
static UNIT3: Mutex<RefCell<Option<unit::Unit<'static, 3>>>> = Mutex::new(RefCell::new(None));

static ACCUMULATED0: AtomicI64 = AtomicI64::new(0);
static ACCUMULATED1: AtomicI64 = AtomicI64::new(0);
static ACCUMULATED2: AtomicI64 = AtomicI64::new(0);
static ACCUMULATED3: AtomicI64 = AtomicI64::new(0);

static LOW_LIMIT0: AtomicI64 = AtomicI64::new(DEFAULT_LOW_LIMIT as i64);
static LOW_LIMIT1: AtomicI64 = AtomicI64::new(DEFAULT_LOW_LIMIT as i64);
static LOW_LIMIT2: AtomicI64 = AtomicI64::new(DEFAULT_LOW_LIMIT as i64);
static LOW_LIMIT3: AtomicI64 = AtomicI64::new(DEFAULT_LOW_LIMIT as i64);

static HIGH_LIMIT0: AtomicI64 = AtomicI64::new(DEFAULT_HIGH_LIMIT as i64);
static HIGH_LIMIT1: AtomicI64 = AtomicI64::new(DEFAULT_HIGH_LIMIT as i64);
static HIGH_LIMIT2: AtomicI64 = AtomicI64::new(DEFAULT_HIGH_LIMIT as i64);
static HIGH_LIMIT3: AtomicI64 = AtomicI64::new(DEFAULT_HIGH_LIMIT as i64);

pub fn install_interrupt_handler(pcnt: &mut Pcnt<'_>) {
    pcnt.set_interrupt_handler(pcnt_interrupt_handler);
}

#[esp_hal::handler]
pub fn pcnt_interrupt_handler() {
    handle_unit_interrupt(&UNIT0, &ACCUMULATED0, &LOW_LIMIT0, &HIGH_LIMIT0);
    handle_unit_interrupt(&UNIT1, &ACCUMULATED1, &LOW_LIMIT1, &HIGH_LIMIT1);
    handle_unit_interrupt(&UNIT2, &ACCUMULATED2, &LOW_LIMIT2, &HIGH_LIMIT2);
    handle_unit_interrupt(&UNIT3, &ACCUMULATED3, &LOW_LIMIT3, &HIGH_LIMIT3);
}

fn handle_unit_interrupt<const UNIT: usize>(
    unit_storage: &'static Mutex<RefCell<Option<unit::Unit<'static, UNIT>>>>,
    accumulated: &'static AtomicI64,
    low_limit: &'static AtomicI64,
    high_limit: &'static AtomicI64,
) {
    critical_section::with(|cs| {
        let mut unit_ref = unit_storage.borrow_ref_mut(cs);

        if let Some(unit) = unit_ref.as_mut() {
            if unit.interrupt_is_set() {
                let events = unit.events();

                if events.high_limit {
                    accumulated.fetch_add(high_limit.load(Ordering::SeqCst), Ordering::SeqCst);
                } else if events.low_limit {
                    accumulated.fetch_add(low_limit.load(Ordering::SeqCst), Ordering::SeqCst);
                }

                unit.reset_interrupt();
            }
        }
    });
}

macro_rules! impl_unit_constructor {
    ($unit_num:literal, $new:ident, $unit_storage:ident, $accumulated:ident, $low_limit:ident, $high_limit:ident) => {
        impl<S> EspPcntEncoder<$unit_num, S>
        where
            S: EncoderSpec,
        {
            pub fn $new(
                unit: unit::Unit<'static, $unit_num>,
                pin_a: impl InputPin + 'static,
                pin_b: impl InputPin + 'static,
                config: PcntConfig,
            ) -> Result<Self, EncoderError> {
                if critical_section::with(|cs| $unit_storage.borrow_ref(cs).is_some()) {
                    return Err(EncoderError::UnitAlreadyConfigured);
                }

                let (counter, pin_a, pin_b) =
                    configure_quadrature_unit(&unit, pin_a, pin_b, config)?;

                $accumulated.store(0, Ordering::SeqCst);
                $low_limit.store(config.low_limit as i64, Ordering::SeqCst);
                $high_limit.store(config.high_limit as i64, Ordering::SeqCst);

                critical_section::with(|cs| {
                    let mut stored = $unit_storage.borrow_ref_mut(cs);
                    if stored.is_some() {
                        return Err(EncoderError::UnitAlreadyConfigured);
                    }

                    stored.replace(unit);
                    if let Some(stored_unit) = stored.as_ref() {
                        stored_unit.listen();
                        stored_unit.resume();
                    }
                    Ok(())
                })?;

                Ok(Self {
                    counter,
                    _pin_a: pin_a,
                    _pin_b: pin_b,
                    _spec: PhantomData,
                })
            }
        }
    };
}

impl_unit_constructor!(0, new_unit0, UNIT0, ACCUMULATED0, LOW_LIMIT0, HIGH_LIMIT0);
impl_unit_constructor!(1, new_unit1, UNIT1, ACCUMULATED1, LOW_LIMIT1, HIGH_LIMIT1);
impl_unit_constructor!(2, new_unit2, UNIT2, ACCUMULATED2, LOW_LIMIT2, HIGH_LIMIT2);
impl_unit_constructor!(3, new_unit3, UNIT3, ACCUMULATED3, LOW_LIMIT3, HIGH_LIMIT3);

fn configure_quadrature_unit<const UNIT: usize>(
    unit: &unit::Unit<'static, UNIT>,
    pin_a: impl InputPin + 'static,
    pin_b: impl InputPin + 'static,
    config: PcntConfig,
) -> Result<(unit::Counter<'static, UNIT>, Input<'static>, Input<'static>), EncoderError> {
    unit.pause();
    unit.set_low_limit(Some(config.low_limit))
        .map_err(|_| EncoderError::InvalidLowLimit)?;
    unit.set_high_limit(Some(config.high_limit))
        .map_err(|_| EncoderError::InvalidHighLimit)?;
    unit.set_filter(config.glitch_filter_cycles)
        .map_err(|_| EncoderError::InvalidFilterThreshold)?;
    unit.clear();
    unit.reset_interrupt();

    let input_config = InputConfig::default().with_pull(config.pull);
    let input_a = Input::new(pin_a, input_config);
    let input_b = Input::new(pin_b, input_config);
    let signal_a = input_a.peripheral_input();
    let signal_b = input_b.peripheral_input();

    let ch0 = &unit.channel0;
    ch0.set_ctrl_signal(signal_a.clone());
    ch0.set_edge_signal(signal_b.clone());
    ch0.set_ctrl_mode(channel::CtrlMode::Reverse, channel::CtrlMode::Keep);
    ch0.set_input_mode(channel::EdgeMode::Increment, channel::EdgeMode::Decrement);

    let ch1 = &unit.channel1;
    ch1.set_ctrl_signal(signal_b);
    ch1.set_edge_signal(signal_a);
    ch1.set_ctrl_mode(channel::CtrlMode::Reverse, channel::CtrlMode::Keep);
    ch1.set_input_mode(channel::EdgeMode::Decrement, channel::EdgeMode::Increment);

    Ok((unit.counter.clone(), input_a, input_b))
}

fn accumulator_for_unit<const UNIT: usize>() -> &'static AtomicI64 {
    match UNIT {
        0 => &ACCUMULATED0,
        1 => &ACCUMULATED1,
        2 => &ACCUMULATED2,
        3 => &ACCUMULATED3,
        _ => unreachable!("ESP32-S3 PCNT exposes units 0 through 3"),
    }
}
