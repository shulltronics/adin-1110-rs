//! An `embedded-hal` driver for the ADIN1110 Single-Pair Ethernet Tranceiver
//! (embedded-hal 1.0.0 alpha implementations)

use embedded_hal_v1::spi::SpiDevice;
use embedded_hal::digital::v2::OutputPin;

use crate::ADIN1110;

impl<SPI, CS> ADIN1110<SPI, CS>
    where SPI: SpiDevice,
          CS: OutputPin,
{

}
