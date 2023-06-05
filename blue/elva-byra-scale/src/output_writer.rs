//! This module is responsible to read data indefinitely (at a set rate) from the given scale that implements the
//! [HX711] trait. Furthermore that data is relayed at a set interval to the given writer [Write],
//! which typically will be a file or stdout.

use chrono::{DateTime, Utc};
use std::error::Error;
use std::io::Write;
use std::os::unix::net::UnixStream;

/// This functions takes any type that implements writer and outputs the read HX711 value.
/// Note that this function (and the entire process) assume the metric system.
/// The output is always written to the same line using \r, which means that
/// the output writer can't be used as a history-file.
pub fn write_weight_to_sock(weight: f32, writer: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    let sample = Sample {
        grams: weight,
        // TODO: set on read
        datetime: Utc::now(),
    };

    let data = format!("\r{}", serde_json::to_string(&sample)?.trim());

    writer.write_all(data.as_bytes())?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Sample {
    /// Time of sample creation
    pub datetime: DateTime<Utc>,

    /// Weight at the given sample time
    pub grams: f32,
}
