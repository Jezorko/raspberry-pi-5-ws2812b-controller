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

pub struct SpiLedStripController {
    leds_count: usize,
    /// Each bit will be spread this many times to accommodate larger color spaces.
    bit_spread: usize,
    /// Raw data to be sent to SPI.
    buffer: Vec<u8>,
    /// SPI device to which the strip is connected.
    spi: Spi,
}

impl LedStripController for SpiLedStripController {
    fn len(&self) -> usize {
        self.leds_count
    }

    fn reset(&mut self) {
        self.set_all(0, 0, 0);
    }

    fn set(&mut self, position: usize, red: u8, green: u8, blue: u8) {
        fn bitmask(value: u8, position: usize) -> bool {
            value & (1 << (7 - position)) != 0
        }

        let mut current_byte_position = position * self.bit_spread * (8 * 3/* 8 bits per color, 3 colors */);

        // TODO: extract this loop
        // set green
        for i in 0..8 {
            let current_bit = bitmask(green, i);
            for spread in 0..self.bit_spread {
                if current_bit {
                    self.buffer[current_byte_position] = 0xF8;
                } else {
                    self.buffer[current_byte_position] = 0xC0
                };
                current_byte_position = current_byte_position + 1;
            }
        }
        // set red
        for i in 0..8 {
            let current_bit = bitmask(red, i);
            for spread in 0..self.bit_spread {
                if current_bit {
                    self.buffer[current_byte_position] = 0xF8;
                } else {
                    self.buffer[current_byte_position] = 0xC0
                };
                current_byte_position = current_byte_position + 1;
            }
        }
        // set blue
        for i in 0..8 {
            let current_bit = bitmask(blue, i);
            for spread in 0..self.bit_spread {
                if current_bit {
                    self.buffer[current_byte_position] = 0xF8;
                } else {
                    self.buffer[current_byte_position] = 0xC0
                };
                current_byte_position = current_byte_position + 1;
            }
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

pub fn create_ws2812b_strip(
    leds_count: usize,
    color_depth_bits: usize,
) -> Result<SpiLedStripController, Box<dyn Error>> {
    let bit_spread;
    if (color_depth_bits == 12) {
        bit_spread = 1;
    } else if (color_depth_bits == 24) {
        bit_spread = 3;
    } else {
        bit_spread = 1;
    }

    let buffer = vec![0; bit_spread * (8 * 3/* 8 bits per color, 3 colors per LED */) * leds_count];
    let spi_speed_khz: u32 = 800;
    let spi_speed_bps: u32 = spi_speed_khz * 1024 * 8; // Convert kHz to bytes per second (TODO: why???)
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, spi_speed_bps, Mode::Mode0)?;
    thread::sleep(Duration::from_millis(100)); // Short delay to ensure device is ready (TODO: needed???)

    Ok(SpiLedStripController {
        leds_count,
        bit_spread,
        buffer,
        spi,
    })
}
