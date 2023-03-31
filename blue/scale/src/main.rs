//! The byra-scale is used to read values from load cells (in conjunction with the HX711 module).
//! This binary has only been tested on the raspberry pi zero (w).
//!
//! # Examples
//!
//! ## Calibrate
//! This command samples reads from the load cells and then outputs the average rate output
//! for both 0kg and 1kg load.
//!
//! ```bash
//! elva-byra-scale --calibrate
//! ```
//!
//! ## Run
//! As a long lived process
//!
//! ```bash
//! elva-byra # Run using settings from ~/.config/byra/settings.toml
//!
//! elva-byra --help
//! ```

use clap::Parser;
use config::Config;
use log::{debug, info};
use rppal::gpio::Gpio;
use simple_logger::SimpleLogger;
use std::env;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Duration;

mod cli_config;

use crate::cli_config::{Args, ServiceConfig};
use elva_byra_lib::hx711::{Config as HXConfig, Gain, Scale, HX711};
use elva_byra_lib::output_writer::stream_weight_to_writer;

static MODULE: &str = "HX711";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let settings_file = PathBuf::from(match args.settings_path {
        Some(file_path) => file_path,
        None => format!(
            "{}/.config/byra/settings.toml",
            env::var("HOME").expect("Failed to read home dir env (HOME)")
        ),
    })
    .canonicalize()
    .expect("Failed to resolve configuration file path");

    let settings = settings_file.to_str();

    if settings.is_none() {
        panic!("failed To read settings from file");
    }

    let settings = settings.unwrap();

    debug!("Trying to read settings from {}", settings);

    let settings = Config::builder()
        .add_source(config::File::with_name(settings))
        .build()
        .unwrap_or_else(|_| panic!("Failed to read settings from {}", settings));
    let service_settings = settings.try_deserialize::<ServiceConfig>().unwrap();

    SimpleLogger::new()
        .with_level(match args.verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Warn,
        })
        .init()
        .unwrap();
    info!("Starting byra-scale, setting up gpio & performing hx711 reset");

    // Initiate gpio & scale
    let gpio = Gpio::new()?;
    let mut scale = Scale::new(HXConfig {
        dt_sck: gpio.get(service_settings.dt_sck)?.into_output(),
        dout: gpio.get(service_settings.dout)?.into_input(),
        kg_1: service_settings.calibration,
        kg_0: service_settings.offset,
        gain: Gain::G128,
    });

    scale.reset();
    info!("{MODULE} reset complete");

    let mut output_writer: Box<dyn Write> = match service_settings.output_file {
        None => Box::new(BufWriter::new(std::io::stdout())),
        Some(f) => Box::new(std::fs::File::create(f)?),
    };

    if args.calibrate {
        let result = scale.calibrate(10);

        info!("\roffset={}\ncalibration={}", result.0, result.1);

        return Ok(());
    }

    stream_weight_to_writer(
        &mut scale,
        service_settings.retry as u32,
        Duration::from_secs(service_settings.backoff),
        &mut output_writer,
    )?;

    Ok(())
}
