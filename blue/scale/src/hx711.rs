use log::info;
use rppal::gpio::{InputPin, Level, OutputPin};
use std::{thread, time::Duration};

pub trait HX711 {
    /// Resets ADC (min 60us), default gain after boot is 128.
    fn reset(&mut self);

    /// Reads 24bits from ADC & sets gain for future com.
    fn read(&mut self, gain: Gain) -> Result<i32, HX711Error>;

    /// Sends a pulse through configured dt_sck pin.
    fn send_pulse(&mut self) -> Result<(), HX711Error>;

    /// Returns true if dout pin is low, which indicates that data is ready for read.
    fn is_ready(&self) -> bool;

    fn sample_avg(&mut self, n: usize) -> f32;

    fn calibrate(&mut self);

    fn get_scale(&mut self) -> f32;

    fn get_offset(&mut self) -> f32;

    fn translate(&self, read: i32) -> f32;
}

#[derive(Debug)]
pub enum HX711Error {
    DOUTNotReady,
    SCKHigh,
    // GenericError,
}

pub struct Config {
    pub dout: InputPin,
    pub dt_sck: OutputPin,
}

pub enum Gain {
    G128 = 1,
    G64,
    G32,
}

impl Scale {
    pub fn new(c: Config) -> Self {
        Self {
            dout: c.dout,
            dt_sck: c.dt_sck,
            scale: 0_f32,
            offset: 0_f32,
        }
    }
}

pub struct Scale {
    dout: InputPin,
    dt_sck: OutputPin,
    scale: f32,
    offset: f32,
}

/// Default implementaiton for a Byra Scale
impl HX711 for Scale {
    fn reset(&mut self) {
        self.dt_sck.set_high();
        thread::sleep(Duration::from_micros(120));
        self.dt_sck.set_low();
    }

    fn read(&mut self, gain: Gain) -> Result<i32, HX711Error> {
        if !self.dout.is_low() {
            return Err(HX711Error::DOUTNotReady);
        }

        let mut buff = 0;

        for _ in 0..24 {
            self.send_pulse()?;
            thread::sleep(Duration::from_nanos(100));
            buff <<= 1;
            buff |= match self.dout.read() {
                Level::Low => 0b0,
                Level::High => 0b1,
            };
        }

        // Sets gain for following reads...
        for _ in 0..match gain {
            Gain::G32 => 3,
            Gain::G64 => 2,
            Gain::G128 => 1,
        } {
            self.send_pulse()?;
        }

        Ok(buff)
    }

    fn send_pulse(&mut self) -> Result<(), HX711Error> {
        if self.dt_sck.is_set_high() {
            return Err(HX711Error::SCKHigh);
        }

        self.dt_sck.set_high();
        self.dt_sck.set_low();

        Ok(())
    }

    fn is_ready(&self) -> bool {
        !self.dout.is_low()
    }

    fn sample_avg(&mut self, n: usize) -> f32 {
        let mut samples = vec![];

        while samples.len() < n {
            if let Ok(d) = self.read(Gain::G128) {
                samples.push(d)
            }
        }

        samples.iter().copied().sum::<i32>() as f32 / n as f32
    }

    fn calibrate(&mut self) {
        info!("Calibrating, remove any wight from the scale");
        thread::sleep(Duration::from_secs(10));
        let kg_0 = self.sample_avg(10);

        info!("Place 1KG on scale");
        thread::sleep(Duration::from_secs(10));

        let kg_1 = self.sample_avg(10);

        self.offset = kg_0;
        self.scale = (kg_1 - kg_0) / 1000_f32;

        info!(
            "Calibration complete kg0={kg_0}, kg1={kg_1}, kg/unit={}",
            self.scale
        );
        thread::sleep(Duration::from_secs(5));
    }

    fn get_scale(&mut self) -> f32 {
        self.scale
    }

    fn get_offset(&mut self) -> f32 {
        self.offset
    }

    /// Transforms given digital value to grams, based on default state x0 & 1kg state.
    fn translate(&self, read: i32) -> f32 {
        // TODO: read from conf
        let x0 = 516580;
        let kg_1 = 538822;
        let diff = kg_1 - x0;
        let points_per_gram = diff / 1000;

        (read - x0) as f32 / points_per_gram as f32
    }
}

// /// Convert 24 bit signed integer to i32
// fn i24_to_i32(x: i32) -> i32 {
//     if x >= 0x800000 {
//         x | !0xFFFFFF
//     } else {
//         x
//     }
// }
