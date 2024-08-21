mod timings;
mod test_extensions;
mod strip;
mod instructions;

use crate::strip::{create_strip, LedController};
use crate::timings::{get_signal_representation_in_bytes, DEFAULT_WS2812B_TIMING_REQUIREMENTS};
use bitvec::order::{BitOrder, Lsb0, Msb0};
use rppal::spi::BitOrder::{LsbFirst, MsbFirst};
use rppal::spi::{Bus, Mode, Segment, SlaveSelect, Spi};

use std::error::Error;
use std::fmt::{Display, Formatter, Write};
use std::thread;
use std::time::Duration;
use rppal::gpio::Gpio;
use crate::instructions::{SPI_INSTRUCTION_READ_STATUS_REGISTER, SPI_INSTRUCTION_WRITE, SPI_INSTRUCTION_WRITE_IN_PROCESS};

/// 8MHz == 125ns
const SPI_CLOCK_SPEED: u32 = 8_000_000;

struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Display for Color {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("R:{} G:{} B:{}", self.red, self.green, self.blue))
    }
}

fn test_pin() -> Result<(), Box<dyn Error>> {
    // testing GPIO 10
    let mut pin = Gpio::new()?.get(10)?.into_output();

    println!("Testing if correct GPIO pin is connected");
    pin.toggle();
    thread::sleep(Duration::from_millis(2_000));
    pin.toggle();
    println!("Testing done!");

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    test_pin()?;

    // Configure the SPI peripheral. The 24AA1024 clocks in data on the first
    // rising edge of the clock signal (SPI mode 0). At 3.3 V, clock speeds of up
    // to 10 MHz are supported.
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1, Mode::Mode0)?;

    let bit_order = spi.bit_order()?;
    println!("bit order is {}", bit_order);

    println!("testing with spi MSB; strip MSB");
    spi.set_bit_order(MsbFirst)?;
    test_strip::<Msb0>(&mut spi)?;

    println!("testing with spi MSB; strip LSB");
    spi.set_bit_order(MsbFirst)?;
    test_strip::<Lsb0>(&mut spi)?;

    println!("testing with spi LSB; strip MSB");
    spi.set_bit_order(LsbFirst)?;
    test_strip::<Msb0>(&mut spi)?;

    println!("testing with spi LSB; strip LSB");
    spi.set_bit_order(LsbFirst)?;
    test_strip::<Lsb0>(&mut spi)?;

    Ok(())
}

pub fn test_strip<DataBitsOrdering>(spi: &mut Spi) -> Result<(), Box<dyn Error>>
where
    DataBitsOrdering: BitOrder,
{
    let mut strip = create_strip::<DataBitsOrdering>(3, get_signal_representation_in_bytes(SPI_CLOCK_SPEED, DEFAULT_WS2812B_TIMING_REQUIREMENTS));

    strip.write_to_spi_blocking(spi)?;
    thread::sleep(Duration::from_secs(1));


    [
        Color { red: 255, green: 0, blue: 0 },
        Color { red: 0, green: 255, blue: 0 },
        Color { red: 0, green: 0, blue: 255 },
        // Color { red: 255, green: 255, blue: 0 },
        // Color { red: 255, green: 0, blue: 255 },
        // Color { red: 0, green: 255, blue: 255 },
        // Color { red: 150, green: 0, blue: 150 },
        // Color { red: 100, green: 240, blue: 150 }
    ].iter().for_each(|color| {
        println!("setting color to {}", color);
        strip.set_all_leds(color.red, color.green, color.blue);
        strip.print_buffer();
        strip.write_to_spi_blocking(spi).unwrap();
        thread::sleep(Duration::from_secs(5));
    });

    Ok(())
}