//! The byra-scale is used to read values from load cells (in conjunction with the HX711 module)
//!
//! # Examples
//!
//! ## Calibrate
//! This command samples a set of read from the load cells and then outputs the average rate output
//! for both 0kg and 1kg load.
//!
//! ```bash
//! elva-byra calibrate
//! ```
//!
//! ## Run
//! As a long lived process
//!
//! ```bash
//! elva-byra
//! ```

use config::Config;
use log::{debug, info};
use rppal::gpio::Gpio;
use simple_logger::SimpleLogger;
use std::io::{BufWriter, Write};
use std::time::Duration;

use byra::output_writer::stream_weight_to_writer;

mod cli_config;

use crate::cli_config::ServiceConfig;
use byra::hx711::{Config as HXConfig, Gain, Scale, HX711};

static MODULE: &str = "HX711";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    // TODO: read config path from cli input or default
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

    let mut output_writer: Box<dyn Write> = match service_settings.file {
        None => Box::new(BufWriter::new(std::io::stdout())),
        Some(f) => Box::new(std::fs::File::open(f)?),
    };

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
            stream_weight_to_writer(
                &mut scale,
                service_settings.retry as u32,
                default_sleep,
                &mut output_writer,
            )?;
        }
    };

    Ok(())
}
