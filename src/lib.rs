//! An `embedded-hal` driver for the ADIN1110 Single-Pair Ethernet Tranceiver

#![no_std]

#[cfg(feature="ehv1")]
mod ehv1;
#[cfg(feature="smoltcp")]
mod smoltcp;

use bitfield_struct::bitfield;

use embedded_hal::spi::FullDuplex;
use embedded_hal::blocking::spi::Write;
use embedded_hal::blocking::spi::transfer::Default as SpiTransferDefault;
//use embedded_hal::blocking::spi::write::Default;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::*;

/// The main driver for the tranceiver
pub struct ADIN1110<SPI, CS> {
    spi: SPI,
    cs: CS,
}

impl<SPI, CS> ADIN1110<SPI, CS>
    where SPI: FullDuplex<u8> + _embedded_hal_blocking_spi_Transfer<u8>,
          CS: OutputPin,
{
    /// Create a new driver instance.
    pub fn new(cs: CS, spi: SPI) -> Self {
        Self {
            spi: spi,
            cs: cs,
        }
    }

    pub fn get_capabilities(&mut self) -> Result<u8, Error> {
        let header = ControlCommandHeader::new()
            .with_parity(false)
            .with_length(0)
            .with_address(Registers::IdVer as u16)
            .with_memmap(0)
            .with_aid(true)
            .with_wnr(false);
        let raw_header: u32 = header.into();
        // the buffer should be 12 bytes:
        //   first 4 are the header,
        //   second 4 will get the header echo from device
        //   third 4 will get the 32 bit register value
        let mut buffer = [0u8; 12];
        buffer[..4].swap_with_slice(&mut raw_header.to_be_bytes());
        match self.spi.transfer(&mut buffer) {
            Ok(_) => Ok(buffer[3]),
            Err(_) => Err(Error::IOError),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IOError,
    OtherError,
}

/// The register map of the device.
#[derive(Debug)]
#[non_exhaustive]
#[repr(u16)]
enum Registers {
    IdVer = 0x00,
    PhyId = 0x01,
    Capability = 0x02,
}

/// The types of headers we can send to control how commands are issued.
/// The bitfield-struct crate makes this really nice.
#[bitfield(u32)]
struct ControlCommandHeader {
    /// Parity bit (unused for writes?)
    parity: bool,
    /// The number of registers to read/write, minus 1
    #[bits(7)]
    length: u8,
    /// The starting address of transmissions
    address: u16,
    /// The memory map to access (0: standard, 1: MAC)
    #[bits(4)]
    memmap: u8,
    /// Address increment disable. (0: post-increment, 1: do not post-increment)
    aid: bool,
    /// Write-not-read. (0: read, 1: write)
    wnr: bool,
    /// Received Header Bad. Ununsed by host, set to 1 in header echo if device detected an error.
    #[bits(default=false)]
    hdrb: bool,
    __: bool,
}

