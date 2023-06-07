use clap::Parser;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ServiceConfig {
    /// Data out pin (23)
    pub dout: u8,

    /// Clock pin (24)
    pub dt_sck: u8,

    /// This should be set to the sensor value when the scale is under no pressure.
    pub offset: u32,

    /// This value should be set to the average sensor read/rate you find when applying
    /// 1KG of pressure to the weight.
    pub calibration: u32,

    /// Sets the read interval for the scale. Hence, how often a
    /// transposed value will be written to output file.
    pub backoff: u64,

    /// Retry limit, if the module doesn't respond before limit the process will panic.
    pub retry: u8,

    /// This is the output file which the scale will stream sensor data to, stdout will be used if
    /// this setting is unset.
    pub output_file: Option<String>,

    pub socket_location: Option<String>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Determines if the process should run calibrate or read -mode
    #[arg(short, long, default_value_t = false)]
    pub calibrate: bool,

    /// Target configuration file, tries to read `~/.config/byra/scale/settings.toml` by default
    #[arg(short, long)]
    pub settings_path: Option<String>,

    /// Toggles verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
