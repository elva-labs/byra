//! The byra-scale is used to read values from load cells (in conjunction with the HX711 module).
//! This binary has only been tested on the raspberry pi zero (w).
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
//! Start a long lived process, readings are pushed to stdout or file (based on given settings).  
//!
//! ```bash
//! elva-byra # Reads settings from `~/.config/byra/settings.toml` by default.
//!
//! elva-byra --help
//! ```
//!
//! ## Example config
//! ```toml
//! # ~/.config/byra/settings.toml
//! dout = 23
//! dt_sck = 24
//! offset = 521703
//! calibration = 545351
//! backoff = 3
//! retry = 3
//!```

use clap::Parser;
use log::info;
use simple_logger::SimpleLogger;
use std::io::{BufWriter, Write};
use std::time::Duration;

mod cli_config;
mod init;

use crate::cli_config::Args;
use crate::init::bootstrap;
use elva_byra_lib::hx711::HX711;
use elva_byra_lib::output_writer::stream_weight_to_writer;

static MODULE: &str = "HX711";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let (settings, byra) = bootstrap(&args).expect("Failed to init byra scale");
    let mut byra = byra;

    SimpleLogger::new()
        .with_level(match args.verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Warn,
        })
        .init()
        .unwrap();
    info!("Starting byra-scale, setting up gpio & performing hx711 reset");
    byra.reset();
    info!("{MODULE} reset complete");

    let mut output_writer: Box<dyn Write> = match settings.output_file {
        None => Box::new(BufWriter::new(std::io::stdout())),
        Some(f) => Box::new(std::fs::File::create(f)?),
    };

    if args.calibrate {
        // TODO: pass N via cli
        let result = byra.calibrate(10);

        info!("\roffset={}\ncalibration={}", result.0, result.1);

        return Ok(());
    }

    stream_weight_to_writer(
        &mut byra,
        &mut output_writer,
        settings.retry,
        Duration::from_secs(settings.backoff),
    )?;

    Ok(())
}
