use config::Config;
use log::{debug, error, info};
use rppal::gpio::Gpio;
use simple_logger::SimpleLogger;
use std::thread;
use std::time::Duration;

mod cli_config;

use crate::cli_config::ServiceConfig;
use byra::hx711::{Config as HXConfig, Gain, Scale, HX711};

static MODULE: &str = "HX711";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    // Read config path via cli flag or default to some folder...
    let settings = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();
    let service_settings = settings.try_deserialize::<ServiceConfig>().unwrap();

    let gpio = Gpio::new()?;
    let mut scale = Scale::new(HXConfig {
        dt_sck: gpio.get(service_settings.dt_sck)?.into_output(),
        dout: gpio.get(service_settings.dout)?.into_input(),
        kg_1: service_settings.calibration,
        kg_0: service_settings.offset,
        gain: Gain::G128,
    });
    let default_sleep = Duration::from_secs(service_settings.backoff);

    info!("Starting byra-scale, setting up gpio & performing hx711 reset");
    debug!("Settings={:?}", service_settings);

    scale.reset();
    info!("{MODULE} reset complete, waiting for first read to become ready...");

    match std::env::args().nth(1) {
        Some(cmd) => match cmd.as_str() {
            "calibrate" => {
                let result = scale.calibrate(10);

                info!("offset={}\ncalibration={}", result.0, result.1);

                return Ok(());
            }
            _ => {
                panic!("The given command is not implemented");
            }
        },
        None => {
            stream_weight_to_file(&mut scale, service_settings.retry as u32, default_sleep)?;
        }
    };

    Ok(())
}

fn stream_weight_to_file(
    scale: &mut impl HX711,
    max_retries: u32,
    timeout: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut retries = 0;

    loop {
        match scale.read() {
            Ok(v) => {
                debug!("digital_value={v}");
                info!("kg={:.2}", scale.translate(v) / 1000_f32);
                // TODO: write output value to stdout or file...
            }
            Err(e) => {
                retries += 1;

                if retries == max_retries {
                    error!("Reach maximum read retries");

                    return Err(e.into());
                }
            }
        }

        thread::sleep(timeout);
    }
}
