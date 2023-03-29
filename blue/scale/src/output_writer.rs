use log::{debug, error};
use std::io::Write;
use std::thread;
use std::time::Duration;

use crate::hx711::HX711;

pub fn stream_weight_to_writer(
    scale: &mut impl HX711,
    max_retries: u32,
    timeout: Duration,
    writer: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut retries = 0;

    loop {
        match scale.read() {
            Ok(v) => {
                let output = format!("kg={:.2}", scale.translate(v) / 1000_f32);
                debug!("digital_value={v}");

                writer.write_all(output.as_bytes())?;
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
