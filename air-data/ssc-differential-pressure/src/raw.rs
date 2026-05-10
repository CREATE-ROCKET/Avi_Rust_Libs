use crate::status::SscStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SscTemperatureResolution {
    Bits8,
    Bits11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawSscSample {
    pub status: SscStatus,
    pub pressure_counts: u16,
    pub temperature_counts: Option<u16>,
    pub temperature_resolution: Option<SscTemperatureResolution>,
    pub raw_bytes: [u8; 4],
    pub raw_len: u8,
}

pub fn decode_pressure(raw: [u8; 2]) -> RawSscSample {
    let mut raw_bytes = [0_u8; 4];
    raw_bytes[..2].copy_from_slice(&raw);

    RawSscSample {
        status: SscStatus::from_bits(raw[0] >> 6),
        pressure_counts: pressure_counts(raw[0], raw[1]),
        temperature_counts: None,
        temperature_resolution: None,
        raw_bytes,
        raw_len: 2,
    }
}

pub fn decode_pressure_and_temperature_8bit(raw: [u8; 3]) -> RawSscSample {
    let mut raw_bytes = [0_u8; 4];
    raw_bytes[..3].copy_from_slice(&raw);

    // Honeywellの8 bit温度はT[10:3]なので、11 bit式で扱えるよう下位3 bitを0埋めする。
    let temperature_counts = (raw[2] as u16) << 3;

    RawSscSample {
        status: SscStatus::from_bits(raw[0] >> 6),
        pressure_counts: pressure_counts(raw[0], raw[1]),
        temperature_counts: Some(temperature_counts),
        temperature_resolution: Some(SscTemperatureResolution::Bits8),
        raw_bytes,
        raw_len: 3,
    }
}

pub fn decode_pressure_and_temperature_11bit(raw: [u8; 4]) -> RawSscSample {
    RawSscSample {
        status: SscStatus::from_bits(raw[0] >> 6),
        pressure_counts: pressure_counts(raw[0], raw[1]),
        temperature_counts: Some(temperature_counts_11bit(raw[2], raw[3])),
        temperature_resolution: Some(SscTemperatureResolution::Bits11),
        raw_bytes: raw,
        raw_len: 4,
    }
}

pub fn decode_raw_sample(raw: [u8; 4]) -> RawSscSample {
    decode_pressure_and_temperature_11bit(raw)
}

fn pressure_counts(msb: u8, lsb: u8) -> u16 {
    ((((msb & 0x3F) as u16) << 8) | lsb as u16) & 0x3FFF
}

fn temperature_counts_11bit(msb: u8, lsb: u8) -> u16 {
    (((msb as u16) << 3) | ((lsb as u16) >> 5)) & 0x07FF
}
