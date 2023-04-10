use std::{env, io, path::PathBuf};

use config::Config;
use elva_byra_lib::hx711::{Config as HXConfig, Gain, Scale};
use log::debug;
use rppal::gpio::Gpio;

use crate::cli_config::{Args, ServiceConfig};

/// Reads settings from given config path or default to `~/.config/byra/settings.toml`.
/// Then initiates dout & dt_sck gpio.
pub fn bootstrap(args: &Args) -> Result<(ServiceConfig, Scale), Box<dyn std::error::Error>> {
    let settings_file = PathBuf::from(match args.settings_path.clone() {
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
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find settings file",
        )));
    }

    let settings = settings.unwrap();

    debug!("Trying to read settings from {}", settings);

    let settings = Config::builder()
        .add_source(config::File::with_name(settings))
        .build()
        .expect("Failed to build settings file from given path")
        .try_deserialize::<ServiceConfig>()
        .expect("Failed to deserialize settings");
    let gpio = Gpio::new()?;
    let scale = Scale::new(HXConfig {
        dt_sck: gpio.get(settings.dt_sck)?.into_output(),
        dout: gpio.get(settings.dout)?.into_input(),
        kg_1: settings.calibration,
        kg_0: settings.offset,
        gain: Gain::G128,
    });

    Ok((settings, scale))
}
