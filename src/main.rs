mod strip;

use embedded_hal::spi::SpiBus;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let mut strip = strip::create_ws2812b_strip(12)?;

    strip.reset();
    strip.commit()?;
    thread::sleep(Duration::from_millis(100));

    for led_index in 0..2 {
        for color_value in 0..255 {
            strip.set(led_index, color_value, 0, color_value);
            strip.commit()?;
            thread::sleep(Duration::from_millis(10));
        }
        strip.set(led_index, 0, 0, 0);
    }

    strip.reset();
    strip.commit()?;

    Ok(())
}
