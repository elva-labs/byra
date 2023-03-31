//! This module includes everything needed to read output from the HX711 and handle the different
//! types of errors that may occur during communication.

use log::info;
use rppal::gpio::{InputPin, Level, OutputPin};
use std::{fmt::Display, thread, time::Duration};

pub trait HX711 {
    /// Resets ADC (min 60us), default gain after boot is 128.
    fn reset(&mut self);

    /// Reads 24bits from ADC & sets gain for future com.
    fn read(&mut self) -> Result<f32, HX711Error>;

    /// Sends a pulse through configured dt_sck pin.
    fn send_pulse(&mut self) -> Result<(), HX711Error>;

    /// Returns true if dout pin is low, which indicates that data is ready for read.
    fn is_ready(&self) -> bool;

    fn calibrate(&mut self, n: usize) -> (usize, usize);

    fn sample(&mut self, n: usize) -> f32;

    fn translate(&self, read: i32) -> f32;
}

/// This struct reflects the two different errors that could occur when we pull data from the HX11.
#[derive(Debug)]
pub struct HX711Error {
    source: HX711ErrorType,
}

impl HX711Error {
    fn new(s: HX711ErrorType) -> Self {
        Self { source: s }
    }
}

#[derive(Debug)]
pub enum HX711ErrorType {
    DoutNotReady,
    SckHigh,
}

impl std::error::Error for HX711Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for HX711Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HX711 err={:?}", self.source)
    }
}

pub struct Config {
    pub dout: InputPin,
    pub dt_sck: OutputPin,
    pub kg_0: u32,
    pub kg_1: u32,
    pub gain: Gain,
}

pub struct Scale {
    dout: InputPin,
    dt_sck: OutputPin,
    offset: f32,
    points_per_gram: f32,
    gain: Gain,
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
            offset: c.kg_0 as f32,
            points_per_gram: (c.kg_1 - c.kg_0) as f32 / 1000_f32,
            gain: c.gain,
        }
    }
}

/// Default implementation for a Byra Scale
impl HX711 for Scale {
    fn reset(&mut self) {
        self.dt_sck.set_high();
        thread::sleep(Duration::from_micros(120));
        self.dt_sck.set_low();
    }

    fn read(&mut self) -> Result<f32, HX711Error> {
        if !self.dout.is_low() {
            return Err(HX711Error::new(HX711ErrorType::DoutNotReady));
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
        for _ in 0..match self.gain {
            Gain::G32 => 3,
            Gain::G64 => 2,
            Gain::G128 => 1,
        } {
            self.send_pulse()?;
        }

        Ok(self.translate(buff))
    }

    fn send_pulse(&mut self) -> Result<(), HX711Error> {
        match self.dt_sck.is_set_high() {
            true => Err(HX711Error::new(HX711ErrorType::SckHigh)),
            false => {
                self.dt_sck.set_high();
                self.dt_sck.set_low();

                Ok(())
            }
        }
    }

    fn is_ready(&self) -> bool {
        !self.dout.is_low()
    }

    fn sample(&mut self, n: usize) -> f32 {
        let mut samples = vec![];

        while samples.len() < n {
            if let Ok(d) = self.read() {
                samples.push(d)
            }
        }

        samples.iter().sum::<f32>() / n as f32
    }

    fn calibrate(&mut self, n: usize) -> (usize, usize) {
        info!("Calibrating, remove any weight from the scale");
        thread::sleep(Duration::from_secs(10));

        let kg_0 = self.sample(n);

        info!("Place 1KG on scale");
        thread::sleep(Duration::from_secs(10));

        let kg_1 = self.sample(n);

        (kg_0 as usize, kg_1 as usize)
    }

    /// Transforms given digital value to grams, based on default state kg_0 & kg_1 state.
    fn translate(&self, read: i32) -> f32 {
        (read as f32 - self.offset) as f32 / self.points_per_gram as f32
    }
}

pub trait MetricOutput {
    fn as_kg(&self) -> f32;
}

impl MetricOutput for f32 {
    fn as_kg(&self) -> f32 {
        self / 1000_f32
    }
}
