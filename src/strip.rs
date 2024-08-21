use crate::instructions::{SPI_INSTRUCTION_READ_STATUS_REGISTER, SPI_INSTRUCTION_WRITE, SPI_INSTRUCTION_WRITE_ENABLE, SPI_INSTRUCTION_WRITE_IN_PROCESS};
use crate::timings::WS2812BSpecification;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use rppal::spi::{Segment, Spi};
use std::error::Error;


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
        let mut led_data_index_in_buffer = 4 + ((position * self.specification.bytes_of_data_per_color) * self.specification.colors_per_led);
        // TODO: translate each byte of color to 16 bytes of signal
        // TODO: set data in buffer
        // TODO: color ordering could be RGB and not GRB etc.
        [green, red, blue].iter().for_each(|color_value| {
            self.specification.color_values[color_value.as_usize()].iter().for_each(|data_byte| {
                self.buffer[led_data_index_in_buffer] = *data_byte;
                led_data_index_in_buffer += 1;
            });
        });

        // self.buffer[led_data_index_in_buffer] = 0x00000000; // TODO: why was this here???
    }

    fn set_all_leds(&mut self, red: u8, green: u8, blue: u8) {
        for led_index in 0..self.len() {
            self.set_led(led_index, red, green, blue);
        }
    }

    fn write_to_spi(&mut self, spi: &mut Spi) -> Result<(), Box<dyn Error>> {
        println!("enabling writes");

        // Set the write enable latch using the WREN instruction. This is required
        // before any data can be written. The write enable latch is automatically
        // reset after a WRITE instruction is successfully executed.
        spi.write(&[SPI_INSTRUCTION_WRITE_ENABLE])?;

        println!("writing buffer to SPI");
        spi.write(&self.buffer[0..self.buffer.len()])?;
        Ok(())
    }

    fn write_to_spi_blocking(&mut self, spi: &mut Spi) -> Result<(), Box<dyn Error>> {
        self.write_to_spi(spi)?;
        // Read the STATUS register by writing the RDSR instruction, and then reading
        // a single byte. Loop until the WIP bit is set to 0, indicating the write
        // operation is completed. transfer_segments() will keep the Slave Select line
        // active until both segments have been transferred.
        println!("reading the STATUS register");
        let mut buffer = [0u8; 1];
        loop {
            spi.transfer_segments(&[
                Segment::with_write(&[SPI_INSTRUCTION_READ_STATUS_REGISTER]),
                Segment::with_read(&mut buffer),
            ])?;

            if buffer[0] & SPI_INSTRUCTION_WRITE_IN_PROCESS == 0 {
                break;
            }
        }
        Ok(())
    }

    fn print_buffer(&self) {
        // for byte_index in 0..self.buffer.len() {
        //     let value = self.buffer[byte_index];
        //     println!("{byte_index:03}: {value:08b} {value:02x}");
        // }
        // self.buffer.iter().for_each(|byte| {
        //     print!("{byte:08b} ")
        // });
        // println!();

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
    // 4 bytes for WRITE instruction and address
    // 2 bytes per bit of data
    // 1 byte per color (R, G, B) == 16 bytes of data per color
    // 3 colors per LED = 48 bytes of data per LED
    // 251 bytes for latch = 4016 bytes of data per latch (zeroed out)
    let mut buffer = vec![0; 4 + (48 * leds_count) + /*4016 TODO: we put only 10 cause of spidev message too long error */ 10];
    println!("buffer size: {}", buffer.len());
    buffer[0] = SPI_INSTRUCTION_WRITE;
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
    }; 2 + (leds_count * colors_per_led) + 1];

    buffer_parts[0] = BufferPart {
        contents: "WRITE instruction".to_string(),
        byte_start_inclusive: 0,
        byte_end_exclusive: 1,
    };
    buffer_parts[1] = BufferPart {
        contents: "Address".to_string(),
        byte_start_inclusive: 1,
        byte_end_exclusive: 4,
    };
    for led_index in 0..leds_count {
        buffer_parts[2 + (led_index * colors_per_led)] = BufferPart {
            contents: format!("LED {led_index:03} G").to_string(),
            byte_start_inclusive: 4 + (led_index * bytes_of_data_per_color * 3),
            byte_end_exclusive: 4 + (led_index * bytes_of_data_per_color * 3) + bytes_of_data_per_color,
        };
        buffer_parts[2 + (led_index * colors_per_led) + 1] = BufferPart {
            contents: format!("LED {led_index:03} R").to_string(),
            byte_start_inclusive: 4 + (led_index * bytes_of_data_per_color * 3) + bytes_of_data_per_color,
            byte_end_exclusive: 4 + (led_index * bytes_of_data_per_color * 3) + (bytes_of_data_per_color * 2),
        };
        buffer_parts[2 + (led_index * colors_per_led) + 2] = BufferPart {
            contents: format!("LED {led_index:03} B").to_string(),
            byte_start_inclusive: 4 + (led_index * bytes_of_data_per_color * 3) + (bytes_of_data_per_color * 2),
            byte_end_exclusive: 4 + (led_index * bytes_of_data_per_color * 3) + (bytes_of_data_per_color * 3),
        };
    }
    buffer_parts[2 + (leds_count * colors_per_led)] = BufferPart {
        contents: "Latch".to_string(),
        byte_start_inclusive: buffer_parts[2 + (leds_count * colors_per_led) - 1].byte_end_exclusive,
        byte_end_exclusive: buffer.len(),
    };

    let specification = WS2812BStripSpecification { leds_count, color_values, buffer_parts, colors_per_led, possible_color_values, bytes_of_data_per_color };
    let mut result = WS2812BStrip { buffer, specification };
    result.reset_leds();
    result
}
