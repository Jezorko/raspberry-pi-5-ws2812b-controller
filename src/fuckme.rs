use std::error::Error;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub fn fuck_me() -> Result<(), Box<dyn Error>> {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?;



    Ok(())
}