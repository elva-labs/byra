use config::Config;
use log::{debug, info, warn};
use rppal::gpio::{Error, Gpio};
use simple_logger::SimpleLogger;
use std::thread;
use std::time::Duration;

use byra::hx711::{Config as HXConfig, Gain, Scale, HX711};

static MODULE: &str = "HX711";

#[derive(serde::Deserialize, Debug)]
struct ServiceConfig {
    /// Data out pin (23)
    pub dout: u8,

    /// Clock pin (24)
    pub dt_sck: u8,

    /// This should be set to the sensor value when the scale is under no pressure.
    pub offset: u32,

    /// This value should be set to the avg.sensor value you find when applying
    /// 1KG of pressure to the weight.
    pub calibration: u32,

    /// Backoff / timeout in seconds, sets the read interval for the scale. Hence, how often a
    /// trasposed value will be written to output file.
    pub backoff: u64,

    /// Retry limit, if the module doesn't respond before limit the process will panic.
    pub retry: u8,
}

fn main() -> Result<(), Error> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();
    let service_settings = settings.try_deserialize::<ServiceConfig>().unwrap();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
    info!("Starting byra-scale, setting up gpio & performing hx711 reset");
    debug!("Settings={:?}", service_settings);

    let gpio = Gpio::new()?;
    let mut scale = Scale::new(HXConfig {
        dt_sck: gpio.get(service_settings.dt_sck)?.into_output(),
        dout: gpio.get(service_settings.dout)?.into_input(),
        kg_1: service_settings.calibration,
        kg_0: service_settings.offset,
    });
    let default_sleep = Duration::from_secs(service_settings.backoff);

    scale.reset();
    info!("{MODULE} reset complete, waiting for first read to become ready...");

    let mut retries = 0;

    while !scale.is_ready() {
        retries += 1;
        info!(
            "{MODULE} is not ready yet..., retires left={}",
            service_settings.retry - retries
        );

        if retries == service_settings.retry {
            panic!("Failed to start {MODULE}, can't continue...")
        }

        thread::sleep(default_sleep);
    }

    info!(
        "Scale is ready, starting to collect values in {} seconds",
        service_settings.backoff
    );
    thread::sleep(default_sleep);

    loop {
        match scale.read(Gain::G128) {
            Ok(v) => {
                debug!("raw_digial_value={v}");
                info!("kg={:.2}", scale.translate(v) / 1000_f32);
                // TODO: write output value to stdout or file...
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }

        thread::sleep(default_sleep);
    }
}
