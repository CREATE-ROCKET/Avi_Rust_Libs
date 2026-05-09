#![no_std]

pub mod conversion;
pub mod error;
#[cfg(feature = "esp32s3-pcnt")]
pub mod esp_pcnt;
pub mod spec;
pub mod traits;

pub use conversion::{
    CountsPerRevolution, counts_to_motor_revolutions, counts_to_output_degrees,
    counts_to_output_revolutions, motor_counts_per_revolution, output_counts_per_revolution,
    output_revolutions_to_counts,
};
pub use error::{EncoderError, Result};
pub use spec::{EncoderSpec, Pololu99To1_25DHp12V};
pub use traits::{
    Direction, Encoder, EncoderCount, EncoderDelta, QuadratureEncoder, ResettableEncoder,
};

#[cfg(feature = "async")]
pub use traits::{EncoderAsyncExt, TimedEncoderSample};
