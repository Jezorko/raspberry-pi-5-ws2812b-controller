// Instruction set.
pub const SPI_INSTRUCTION_WRITE: u8 = 0b0010; // Write data, starting at the selected address.
pub const SPI_INSTRUCTION_READ_STATUS_REGISTER: u8 = 0b0101; // Read the STATUS register.
pub const SPI_INSTRUCTION_WRITE_ENABLE: u8 = 0b0110; // Set the write enable latch (enable write operations).

pub const SPI_INSTRUCTION_WRITE_IN_PROCESS: u8 = 1; // Write-In-Process bit mask for the STATUS register.