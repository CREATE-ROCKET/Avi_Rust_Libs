use crate::status::SscStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawSscSample {
    pub status: SscStatus,
    pub pressure_counts: u16,
    pub temperature_counts: u16,
    pub raw_bytes: [u8; 4],
}

pub fn decode_raw_sample(raw: [u8; 4]) -> RawSscSample {
    let status = SscStatus::from_bits(raw[0] >> 6);
    let pressure_counts = ((((raw[0] & 0x3F) as u16) << 8) | raw[1] as u16) & 0x3FFF;
    let temperature_counts = (((raw[2] as u16) << 3) | ((raw[3] as u16) >> 5)) & 0x07FF;

    RawSscSample {
        status,
        pressure_counts,
        temperature_counts,
        raw_bytes: raw,
    }
}
