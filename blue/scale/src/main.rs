use log::{error, info, warn};
use rppal::gpio::{Error, Gpio};
use std::time::Duration;
use std::{thread, time};

use byra::hx711::{Config, Gain, Scale, HX711};

// TODO: set these pins via cfg.
const DOUT: u8 = 0;
const DT_SCK: u8 = 0;
const RETRY_LIMIT: u8 = 10;
static MODULE: &str = "HX711";

fn main() -> Result<(), Error> {
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

        thread::sleep(Duration::from_secs(5));
    }

    match scale.read(Gain::G128) {
        Ok(_) => {}
        Err(e) => {
            error!(
                "Failed to read data from {MODULE}::{:?}, subsequent reads will probably fail.",
                e
            );
        }
    };

    loop {
        match scale.read(Gain::G128) {
            Ok(v) => {
                info!("Received raw value={}", v);
            }
            Err(e) => {
                warn!("Failed to read from {MODULE}::{:?}", e)
            }
        }

        thread::sleep(time::Duration::from_secs(5));
    }
}
