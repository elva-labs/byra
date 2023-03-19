use rppal::gpio::{InputPin, Level, OutputPin};
use std::{thread, time::Duration};

pub trait HX711 {
    /// Resets ADC (min 60us), default gain after boot is 128.
    fn reset(&mut self);

    /// Reads 24bits from ADC & sets gain for future com.
    fn read(&mut self, gain: Gain) -> Result<u32, HX711Error>;

    /// Sends a pulse through configured dt_sck pin.
    fn send_pulse(&mut self) -> Result<(), HX711Error>;

    /// Returns true if dout pin is low, which indicates that data is ready for read.
    fn is_ready(&self) -> bool;
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
        }
    }
}

pub struct Scale {
    dout: InputPin,
    dt_sck: OutputPin,
}

/// Default implementaiton for a Byra Scale
impl HX711 for Scale {
    fn reset(&mut self) {
        self.dt_sck.set_high();
        thread::sleep(Duration::from_micros(120));
        self.dt_sck.set_low();
    }

    fn read(&mut self, gain: Gain) -> Result<u32, HX711Error> {
        if !self.dout.is_low() {
            return Err(HX711Error::DOUTNotReady);
        }

        let mut buff = 0_u32;

        for _ in 0..25 {
            self.send_pulse()?;

            // Shift each read into result buff
            // Might need to wait 0.1 m_u, to get a stable read.
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
}
