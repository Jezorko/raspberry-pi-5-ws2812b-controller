use crate::instructions::SPI_INSTRUCTION_WRITE;
use crate::timings::WS2812BSpecification;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use rppal::spi::Spi;
use std::error::Error;
use std::thread;
use std::time::Duration;

pub trait LedController {
    fn len(&self) -> usize;
    fn reset_leds(&mut self);
    fn set_led(&mut self, position: usize, red: u8, green: u8, blue: u8);
    fn set_all_leds(&mut self, red: u8, green: u8, blue: u8);
    fn write_to_spi(&mut self, spi: &mut Spi) -> Result<(), Box<dyn Error>>;
    fn write_to_spi_blocking(&mut self, spi: &mut Spi) -> Result<(), Box<dyn Error>>;
    fn print_buffer(&self);
}

/// Describes a part of the buffer (for debugging purposes).
#[derive(Clone)]
struct BufferPart {
    /// What this part of the buffer contains.
    contents: String,
    /// Start index of the buffer.
    byte_start_inclusive: usize,
    /// End index of the buffer.
    byte_end_exclusive: usize,
}

struct WS2812BStripSpecification {
    /// The size of LED strip.
    leds_count: usize,
    /// Data representing each color value.
    color_values: Vec<Vec<u8>>,
    /// Description of the buffer contents.
    buffer_parts: Vec<BufferPart>,
    /// How many colors can one LED represent (e.g. 3 for RGB).
    colors_per_led: usize,
    /// How many colors values can be represented (usually values are in range 0 to 255 inclusive).
    possible_color_values: usize,
    /// How many bytes of data are necessary to represent each color.
    bytes_of_data_per_color: usize,
}

struct WS2812BStrip {
    /// Raw data to be sent to SPI.
    buffer: Vec<u8>,
    /// Description of strip parameters.
    specification: WS2812BStripSpecification,
}

impl LedController for WS2812BStrip {
    fn len(&self) -> usize {
        self.specification.leds_count
    }

    fn reset_leds(&mut self) {
        self.set_all_leds(0, 0, 0);
    }

    fn set_led(&mut self, position: usize, red: u8, green: u8, blue: u8) {
        let mut led_data_index_in_buffer = ((position * self.specification.bytes_of_data_per_color) * self.specification.colors_per_led);
        [green, red, blue].iter().for_each(|color_value| {
            self.specification.color_values[color_value.as_usize()].iter().for_each(|data_byte| {
                self.buffer[led_data_index_in_buffer] = *data_byte;
                led_data_index_in_buffer += 1;
            });
        });
    }

    fn set_all_leds(&mut self, red: u8, green: u8, blue: u8) {
        for led_index in 0..self.len() {
            self.set_led(led_index, red, green, blue);
        }
    }

    fn write_to_spi(&mut self, spi: &mut Spi) -> Result<(), Box<dyn Error>> {
        println!("writing buffer to SPI");
        spi.write(&self.buffer[0..self.buffer.len()])?;
        Ok(())
    }

    fn write_to_spi_blocking(&mut self, spi: &mut Spi) -> Result<(), Box<dyn Error>> {
        self.write_to_spi(spi)?;
        thread::sleep(Duration::from_secs(1));
        Ok(())
    }

    fn print_buffer(&self) {
        println!("Buffer contents:");

        let mut longest_name_length: usize = 0;
        self.specification.buffer_parts.iter().for_each(|buffer_part| {
            if buffer_part.contents.len() > longest_name_length {
                longest_name_length = buffer_part.contents.len();
            }
        });

        let mut longest_buffer_part_bytes: usize = 0;
        self.specification.buffer_parts.iter().for_each(|buffer_part| {
            if buffer_part.byte_end_exclusive - buffer_part.byte_start_inclusive > longest_buffer_part_bytes {
                longest_buffer_part_bytes = buffer_part.byte_end_exclusive - buffer_part.byte_start_inclusive;
            }
        });

        print!("{: <1$}             ", "", longest_name_length);
        for byte_index in 0..longest_buffer_part_bytes {
            print!("{byte_index: <9}");
        }
        println!();

        self.specification.buffer_parts.iter().for_each(|buffer_part| {
            print!("{: <3$} ({:03}..{:03}): ", buffer_part.contents, buffer_part.byte_start_inclusive, buffer_part.byte_end_exclusive, longest_name_length);
            for byte_index in buffer_part.byte_start_inclusive..buffer_part.byte_end_exclusive {
                let byte = self.buffer[byte_index];
                if byte_index != buffer_part.byte_start_inclusive { print!(" "); }
                print!("{byte:08b}")
            }
            println!()
        });
    }
}

pub fn create_strip<DataBitsOrdering>(leds_count: usize, specification: WS2812BSpecification) -> impl LedController
where
    DataBitsOrdering: BitOrder,
{
    // 2 bytes per bit of data
    // 1 byte per color (R, G, B) == 16 bytes of data per color
    // 3 colors per LED = 48 bytes of data per LED
    // 251 bytes for latch = 4016 bytes of data per latch (zeroed out)
    let mut buffer = vec![0; (48 * leds_count) + /*4016 TODO: we put only 10 cause of spidev message too long error */ 10];
    println!("buffer size: {}", buffer.len());
    let colors_per_led = 3;
    let bytes_of_data_per_color = 16;
    let possible_color_values = 256;

    // TODO: populate based on signal specification.
    let mut color_values: Vec<Vec<u8>> = vec![vec![0; bytes_of_data_per_color]; possible_color_values];

    for color_value in 0..possible_color_values {
        let color_value_as_array = &[color_value.as_u8()];
        let bits = color_value_as_array.view_bits::<DataBitsOrdering>();
        for (bit_position, bit) in bits.iter().enumerate() {
            // TODO: this may be more or fewer than two bytes, should be determined from specification
            if bit.as_bool() {
                color_values[color_value][bit_position * 2] = specification.zero_code[0];
                color_values[color_value][(bit_position * 2) + 1] = specification.zero_code[1];
            } else {
                color_values[color_value][bit_position * 2] = specification.one_code[0];
                color_values[color_value][(bit_position * 2) + 1] = specification.one_code[1];
            }
        }
    }

    // Silly stuff just for debugging
    let mut buffer_parts: Vec<BufferPart> = vec![BufferPart {
        contents: "".to_string(),
        byte_start_inclusive: 0,
        byte_end_exclusive: 0
    }; (leds_count * colors_per_led) + 1];

    for led_index in 0..leds_count {
        buffer_parts[led_index * colors_per_led] = BufferPart {
            contents: format!("LED {led_index:03} G").to_string(),
            byte_start_inclusive: led_index * bytes_of_data_per_color * 3,
            byte_end_exclusive: (led_index * bytes_of_data_per_color * 3) + bytes_of_data_per_color,
        };
        buffer_parts[(led_index * colors_per_led) + 1] = BufferPart {
            contents: format!("LED {led_index:03} R").to_string(),
            byte_start_inclusive: (led_index * bytes_of_data_per_color * 3) + bytes_of_data_per_color,
            byte_end_exclusive: (led_index * bytes_of_data_per_color * 3) + (bytes_of_data_per_color * 2),
        };
        buffer_parts[(led_index * colors_per_led) + 2] = BufferPart {
            contents: format!("LED {led_index:03} B").to_string(),
            byte_start_inclusive: (led_index * bytes_of_data_per_color * 3) + (bytes_of_data_per_color * 2),
            byte_end_exclusive: (led_index * bytes_of_data_per_color * 3) + (bytes_of_data_per_color * 3),
        };
    }
    buffer_parts[(leds_count * colors_per_led)] = BufferPart {
        contents: "Latch".to_string(),
        byte_start_inclusive: buffer_parts[(leds_count * colors_per_led) - 1].byte_end_exclusive,
        byte_end_exclusive: buffer.len(),
    };

    let specification = WS2812BStripSpecification { leds_count, color_values, buffer_parts, colors_per_led, possible_color_values, bytes_of_data_per_color };
    let mut result = WS2812BStrip { buffer, specification };
    result.reset_leds();
    result
}
