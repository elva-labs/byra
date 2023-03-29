#[derive(serde::Deserialize, Debug)]
pub struct ServiceConfig {
    /// Data out pin (23)
    pub dout: u8,

    /// Clock pin (24)
    pub dt_sck: u8,

    /// This should be set to the sensor value when the scale is under no pressure.
    pub offset: u32,

    /// This value should be set to the avg.sensor value you find when applying
    /// 1KG of pressure to the weight.
    pub calibration: u32,

    /// Sets the read interval for the scale. Hence, how often a
    /// transposed value will be written to output file.
    pub backoff: u64,

    /// Retry limit, if the module doesn't respond before limit the process will panic.
    pub retry: u8,

    /// File
    pub file: Option<String>,
}
