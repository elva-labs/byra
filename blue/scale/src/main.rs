use log::{debug, info, warn};
use rppal::gpio::{Error, Gpio};
use simple_logger::SimpleLogger;
use std::time::Duration;
use std::{thread, time};

use byra::hx711::{Config, Gain, Scale, HX711};

// TODO: set these pins via cfg.
const DOUT: u8 = 23;
const DT_SCK: u8 = 24;
const RETRY_LIMIT: u8 = 10;
const DEFAULT_TIMEOUT_SECONDS: u64 = 5;
const READ_INTERVAL: u64 = 1;
static MODULE: &str = "HX711";

fn main() -> Result<(), Error> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
    info!("Starting byra-scale, setting up gpio & performing hx711 reset");

    let gpio = Gpio::new()?;
    let mut scale = Scale::new(Config {
        dt_sck: gpio.get(DT_SCK)?.into_output(),
        dout: gpio.get(DOUT)?.into_input(),
    });

    scale.reset();
    info!("{MODULE} reset complete, waiting for first read to become ready...");

    let mut retries = 0;

    while !scale.is_ready() {
        retries += 1;
        info!(
            "{MODULE} is not ready yet..., retires left={}",
            RETRY_LIMIT - retries
        );

        if retries == RETRY_LIMIT {
            panic!("Failed to start {MODULE}, can't continue...")
        }

        thread::sleep(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS));
    }

    info!(
        "Scale is ready, starting to collect values in {} seconds",
        DEFAULT_TIMEOUT_SECONDS
    );
    thread::sleep(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS));

    loop {
        match scale.read(Gain::G128) {
            Ok(v) => {
                debug!("raw_digial_value={v}");
                info!("kg={:.2}", scale.translate(v) / 1000_f32);
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }

        thread::sleep(time::Duration::from_secs(READ_INTERVAL));
    }
}
