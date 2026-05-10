#![no_std]

use core::marker::PhantomData;

pub mod error;
pub mod raw;
pub mod sample;
pub mod spec;
pub mod status;

pub use error::SscError;
pub use raw::RawSscSample;
pub use raw::decode_raw_sample;
pub use sample::DifferentialPressureSample;
pub use spec::SscPressureSpec;
pub use spec::Sscdrrn005pd2a5;
pub use status::SscStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutOfRangePolicy {
    Error,
    Clamp,
    Allow,
}

pub struct SscDifferentialPressure<I2C, Spec> {
    i2c: I2C,
    address: u8,
    out_of_range_policy: OutOfRangePolicy,
    _spec: PhantomData<Spec>,
}

impl<I2C, Spec> SscDifferentialPressure<I2C, Spec>
where
    I2C: embedded_hal_async::i2c::I2c,
    Spec: SscPressureSpec,
{
    pub fn new(i2c: I2C) -> Self {
        Self::new_with_address(i2c, Spec::ADDRESS)
    }

    pub fn new_with_address(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            out_of_range_policy: OutOfRangePolicy::Error,
            _spec: PhantomData,
        }
    }

    pub fn set_out_of_range_policy(&mut self, policy: OutOfRangePolicy) {
        self.out_of_range_policy = policy;
    }

    pub async fn read_raw_allow_stale(&mut self) -> Result<RawSscSample, SscError<I2C::Error>> {
        let mut raw = [0_u8; 4];
        self.i2c
            .read(self.address, &mut raw)
            .await
            .map_err(SscError::I2c)?;
        Ok(decode_raw_sample(raw))
    }

    pub async fn read_sample(
        &mut self,
    ) -> Result<DifferentialPressureSample, SscError<I2C::Error>> {
        let raw = self.read_raw_allow_stale().await?;

        match raw.status {
            SscStatus::Normal => {}
            SscStatus::CommandMode => return Err(SscError::CommandMode),
            SscStatus::StaleData => return Err(SscError::StaleData),
            SscStatus::DiagnosticFault => return Err(SscError::DiagnosticFault),
        }

        if Spec::OUTPUT_MAX_COUNTS <= Spec::OUTPUT_MIN_COUNTS {
            return Err(SscError::InvalidTransferFunction);
        }

        let counts = match self.out_of_range_policy {
            OutOfRangePolicy::Error => {
                if raw.pressure_counts < Spec::OUTPUT_MIN_COUNTS
                    || raw.pressure_counts > Spec::OUTPUT_MAX_COUNTS
                {
                    return Err(SscError::PressureCountsOutOfRange {
                        counts: raw.pressure_counts,
                    });
                }
                raw.pressure_counts
            }
            OutOfRangePolicy::Clamp => raw
                .pressure_counts
                .clamp(Spec::OUTPUT_MIN_COUNTS, Spec::OUTPUT_MAX_COUNTS),
            OutOfRangePolicy::Allow => raw.pressure_counts,
        };

        let pressure_pa = pressure_counts_to_pa::<Spec>(counts);
        let temperature_c = if Spec::HAS_COMPENSATED_TEMPERATURE {
            Some(temperature_counts_to_c(raw.temperature_counts))
        } else {
            None
        };

        Ok(DifferentialPressureSample {
            differential_pressure_pa: pressure_pa,
            temperature_c,
            status: raw.status,
            raw_pressure_counts: raw.pressure_counts,
            raw_temperature_counts: raw.temperature_counts,
        })
    }

    pub fn release(self) -> I2C {
        self.i2c
    }
}

fn pressure_counts_to_pa<Spec>(counts: u16) -> f32
where
    Spec: SscPressureSpec,
{
    let output_span = (Spec::OUTPUT_MAX_COUNTS - Spec::OUTPUT_MIN_COUNTS) as f32;
    let pressure_span = Spec::P_MAX_PA - Spec::P_MIN_PA;
    (counts as f32 - Spec::OUTPUT_MIN_COUNTS as f32) * pressure_span / output_span + Spec::P_MIN_PA
}

fn temperature_counts_to_c(counts: u16) -> f32 {
    (counts as f32) * 200.0 / 2047.0 - 50.0
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::future::Future;
    use core::task::Context;
    use core::task::Poll;
    use embedded_hal::i2c::ErrorKind;
    use embedded_hal::i2c::ErrorType;
    use embedded_hal::i2c::Operation;
    use embedded_hal_async::i2c::I2c;
    use std::boxed::Box;
    use std::sync::Arc;
    use std::task::Wake;
    use std::task::Waker;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DummyError {}

    impl embedded_hal::i2c::Error for DummyError {
        fn kind(&self) -> ErrorKind {
            ErrorKind::Other
        }
    }

    struct DummyI2c {
        response: [u8; 4],
    }

    impl ErrorType for DummyI2c {
        type Error = DummyError;
    }

    impl I2c for DummyI2c {
        async fn transaction(
            &mut self,
            _address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            for operation in operations {
                if let Operation::Read(read) = operation {
                    read.copy_from_slice(&self.response[..read.len()]);
                }
            }
            Ok(())
        }
    }

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn block_on<F>(future: F) -> F::Output
    where
        F: Future,
    {
        let waker = Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        let mut future = Box::pin(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    struct InvalidTransferSpec;

    impl SscPressureSpec for InvalidTransferSpec {
        const ADDRESS: u8 = 0x28;
        const P_MIN_PA: f32 = -1.0;
        const P_MAX_PA: f32 = 1.0;
        const OUTPUT_MIN_COUNTS: u16 = 100;
        const OUTPUT_MAX_COUNTS: u16 = 100;
        const HAS_COMPENSATED_TEMPERATURE: bool = false;
    }

    #[test]
    fn decodes_normal_raw_bytes() {
        let sample = decode_raw_sample([0x12, 0x34, 0xAA, 0xA0]);

        assert_eq!(sample.status, SscStatus::Normal);
        assert_eq!(sample.pressure_counts, 0x1234);
        assert_eq!(sample.temperature_counts, 0x0555);
        assert_eq!(sample.raw_bytes, [0x12, 0x34, 0xAA, 0xA0]);
    }

    #[test]
    fn decodes_stale_status() {
        let sample = decode_raw_sample([0x80, 0x00, 0x00, 0x00]);

        assert_eq!(sample.status, SscStatus::StaleData);
    }

    #[test]
    fn extracts_pressure_counts_as_fourteen_bits() {
        let sample = decode_raw_sample([0xFF, 0xFF, 0x00, 0x00]);

        assert_eq!(sample.pressure_counts, 0x3FFF);
    }

    #[test]
    fn extracts_temperature_counts_as_eleven_bits() {
        let sample = decode_raw_sample([0x00, 0x00, 0xFF, 0xFF]);

        assert_eq!(sample.temperature_counts, 0x07FF);
    }

    #[test]
    fn converts_output_min_counts_to_min_pressure() {
        let pressure = pressure_counts_to_pa::<Sscdrrn005pd2a5>(Sscdrrn005pd2a5::OUTPUT_MIN_COUNTS);

        assert!((pressure - Sscdrrn005pd2a5::P_MIN_PA).abs() < 0.001);
    }

    #[test]
    fn converts_output_max_counts_to_max_pressure() {
        let pressure = pressure_counts_to_pa::<Sscdrrn005pd2a5>(Sscdrrn005pd2a5::OUTPUT_MAX_COUNTS);

        assert!((pressure - Sscdrrn005pd2a5::P_MAX_PA).abs() < 0.001);
    }

    #[test]
    fn invalid_transfer_function_returns_error() {
        let i2c = DummyI2c {
            response: [0x00, 100, 0x00, 0x00],
        };
        let mut sensor = SscDifferentialPressure::<_, InvalidTransferSpec>::new(i2c);
        let result = block_on(sensor.read_sample());

        assert_eq!(result, Err(SscError::InvalidTransferFunction));
    }

    #[test]
    fn allow_policy_converts_counts_below_output_min() {
        let i2c = DummyI2c {
            response: [0x00, 0x00, 0x00, 0x00],
        };
        let mut sensor = SscDifferentialPressure::<_, Sscdrrn005pd2a5>::new(i2c);
        sensor.set_out_of_range_policy(OutOfRangePolicy::Allow);
        let result = block_on(sensor.read_sample()).unwrap();

        assert!(result.differential_pressure_pa < Sscdrrn005pd2a5::P_MIN_PA);
    }
}
