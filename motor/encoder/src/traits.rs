#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncoderCount(pub i64);

impl EncoderCount {
    pub const fn new(counts: i64) -> Self {
        Self(counts)
    }

    pub const fn counts(self) -> i64 {
        self.0
    }
}

impl From<i64> for EncoderCount {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Reverse,
    #[default]
    Stopped,
}

impl Direction {
    pub const fn from_delta(delta: i64) -> Self {
        if delta > 0 {
            Self::Forward
        } else if delta < 0 {
            Self::Reverse
        } else {
            Self::Stopped
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EncoderDelta {
    pub previous: EncoderCount,
    pub current: EncoderCount,
}

impl EncoderDelta {
    pub const fn new(previous: EncoderCount, current: EncoderCount) -> Self {
        Self { previous, current }
    }

    pub const fn counts(self) -> i64 {
        self.current.0 - self.previous.0
    }

    pub const fn direction(self) -> Direction {
        Direction::from_delta(self.counts())
    }
}

pub trait Encoder {
    type Error;

    fn count(&self) -> core::result::Result<EncoderCount, Self::Error>;
}

pub trait ResettableEncoder: Encoder {
    fn reset(&mut self) -> core::result::Result<(), Self::Error>;
}

pub trait QuadratureEncoder: Encoder {}

#[cfg(feature = "async")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedEncoderSample {
    pub count: EncoderCount,
    pub at: embassy_time::Instant,
}

#[cfg(feature = "async")]
impl TimedEncoderSample {
    pub fn now(count: EncoderCount) -> Self {
        Self {
            count,
            at: embassy_time::Instant::now(),
        }
    }
}

#[cfg(feature = "async")]
#[allow(async_fn_in_trait)]
pub trait EncoderAsyncExt: Encoder {
    async fn sample_after(
        &self,
        delay: embassy_time::Duration,
    ) -> core::result::Result<TimedEncoderSample, Self::Error> {
        embassy_time::Timer::after(delay).await;
        self.count().map(TimedEncoderSample::now)
    }

    async fn wait_for_change(
        &self,
        poll_interval: embassy_time::Duration,
    ) -> core::result::Result<TimedEncoderSample, Self::Error> {
        self.wait_for_delta(1, poll_interval).await
    }

    async fn wait_for_delta(
        &self,
        minimum_delta: u64,
        poll_interval: embassy_time::Duration,
    ) -> core::result::Result<TimedEncoderSample, Self::Error> {
        let start = self.count()?.counts();

        loop {
            embassy_time::Timer::after(poll_interval).await;
            let current = self.count()?;

            if current.counts().abs_diff(start) >= minimum_delta {
                return Ok(TimedEncoderSample::now(current));
            }
        }
    }
}

#[cfg(feature = "async")]
impl<T> EncoderAsyncExt for T where T: Encoder + ?Sized {}
