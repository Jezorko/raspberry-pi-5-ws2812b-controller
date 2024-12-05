use embedded_hal::spi::SpiBus;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::error::Error;
use std::thread;
use std::time::Duration;

pub trait LedStripController {
    /// Size of the LED Strip.
    fn len(&self) -> usize;
    /// Turn all LEDs off.
    fn reset(&mut self);
    /// Set the LED at given position to the desired RGB value.
    fn set(&mut self, position: usize, red: u8, green: u8, blue: u8);
    /// Set all LEDs in the strip to the desired RGB value.
    fn set_all(&mut self, red: u8, green: u8, blue: u8);
    /// Writes data to the LED strip.
    fn commit(&mut self) -> Result<(), Box<dyn Error>>;
}

struct SpiLedStripController {
    leds_count: usize,
    /// Raw data to be sent to SPI.
    buffer: Vec<u8>,
    /// SPI device to which the strip is connected.
    spi: Spi,
}

impl LedStripController for SpiLedStripController {
    fn len(&self) -> usize {
        self.buffer.len() / (8 * 3/* 8 bits per color, 3 colors */)
    }

    fn reset(&mut self) {
        self.set_all(0, 0, 0);
    }

    fn set(&mut self, position: usize, red: u8, green: u8, blue: u8) {
        fn bitmask(value: u8, position: usize) -> bool {
            value & (1 << (7 - position)) != 0
        }

        let mut current_byte_position = position * (8 * 3/* 8 bits per color, 3 colors */);

        // TODO: extract this loop
        // set green
        for i in 0..8 {
            if (bitmask(green, i)) {
                self.buffer[current_byte_position] = 0xF8;
            } else {
                self.buffer[current_byte_position] = 0xC0
            };
            current_byte_position = current_byte_position + 1;
        }
        // set red
        for i in 0..8 {
            if (bitmask(red, i)) {
                self.buffer[current_byte_position] = 0xF8;
            } else {
                self.buffer[current_byte_position] = 0xC0
            };
            current_byte_position = current_byte_position + 1;
        }
        // set blue
        for i in 0..8 {
            if (bitmask(blue, i)) {
                self.buffer[current_byte_position] = 0xF8;
            } else {
                self.buffer[current_byte_position] = 0xC0
            };
            current_byte_position = current_byte_position + 1;
        }
    }

    fn set_all(&mut self, red: u8, green: u8, blue: u8) {
        for led_index in 0..self.len() {
            self.set(led_index, red, green, blue);
        }
    }

    fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        self.spi.write(&self.buffer[0..self.buffer.len()])?;
        self.spi.flush()?;
        Ok(())
    }
}

fn create_ws2812b_strip(leds_count: usize) -> Result<impl LedStripController, Box<dyn Error>> {
    let mut buffer = vec![0; (8 * 3/* 8 bits per color, 3 colors per LED */) * leds_count];
    let spi_speed_khz: u32 = 800;
    let spi_speed_bps: u32 = spi_speed_khz * 1024 * 8; // Convert kHz to bytes per second (TODO: why???)
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, spi_speed_bps, Mode::Mode0)?;
    thread::sleep(Duration::from_millis(100)); // Short delay to ensure device is ready (TODO: needed???)

    Ok(SpiLedStripController {
        leds_count,
        buffer,
        spi,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut strip = create_ws2812b_strip(12)?;

    strip.set_all(0, 255, 0);
    strip.commit()?;
    thread::sleep(Duration::from_secs(1));

    strip.set_all(255, 0, 0);
    strip.commit()?;
    thread::sleep(Duration::from_secs(1));

    strip.set_all(0, 255, 255);
    strip.commit()?;
    thread::sleep(Duration::from_secs(1));

    strip.reset();
    strip.commit()?;
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
