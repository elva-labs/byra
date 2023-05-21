//! This module is responsible to read data indefinitely (at a set rate) from the given scale that implements the
//! [HX711] trait. Furthermore that data is relayed at a set interval to the given writer [Write],
//! which typically will be a file or stdout.

use chrono::{DateTime, Utc};
use log::error;
use std::io::Write;
use std::thread;
use std::time::Duration;

use crate::hx711::HX711;

/// This functions takes any type that implements writer and outputs the read HX711 value.
/// Note that this function (and the entire process) assume the metric system.
/// The output is always written to the same line using \r, which means that
/// the output writer can't be used as a history-file.
pub fn stream_weight_to_writer(
    scale: &mut impl HX711,
    writer: &mut dyn Write,
    max_retries: u8,
    rate: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut retries = 0;

    loop {
        match scale.read() {
            Ok(output) => {
                let sample = Sample {
                    grams: output,
                    datetime: Utc::now(),
                };
                let data = format!(
                    "\r{}",
                    serde_json::to_string(&sample)
                        .expect("Failed to parse sample to JSON")
                        .trim()
                );

                log::debug!("{:?}", sample);
                writer
                    .write_all(data.as_bytes())
                    .expect("Failed to write data to given writer");
                writer.flush().expect("Failed to flush stream");
                retries = 0;
            }
            Err(e) => {
                retries += 1;

                if retries == max_retries {
                    error!("Reach maximum read retries");

                    return Err(e.into());
                }
            }
        }

        thread::sleep(rate);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Sample {
    /// Time of sample creation
    pub datetime: DateTime<Utc>,

    /// Weight at the given sample time
    pub grams: f32,
}
