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
pub struct ADIN1110<SPI> {
    spi: SPI,
    //cs: CS,
}

impl<SPI> ADIN1110<SPI>
    where SPI: FullDuplex<u8> + _embedded_hal_blocking_spi_Transfer<u8>,
          //CS: OutputPin,
{
    /// Create a new driver instance.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi: spi,
            //cs: cs,
        }
    }

    pub fn get_idver(&mut self) -> Result<u32, Error> {
        let mut header = ControlCommandHeader::new()
            .with_length(0)
            .with_address(Registers::IdVer as u16)
            .with_memmap(0)
            .with_aid(true)
            .with_wnr(false)
            .calculate_parity();
        let raw_header: u32 = header.into();
        let mut buffer = [0u8; 12];
        buffer[..4].swap_with_slice(&mut raw_header.to_be_bytes());
        match self.spi.transfer(&mut buffer) {
            Ok(_) => {
                let v: [u8; 4] = buffer[8..].try_into().unwrap();
                Ok(u32::from_be_bytes(v))
            },
            Err(_) => Err(Error::IOError),
        }
    }

    pub fn get_phyid(&mut self) -> Result<u32, Error> {
        let mut header = ControlCommandHeader::new()
            .with_length(0)
            .with_address(Registers::PhyId as u16)
            .with_memmap(0)
            .with_aid(true)
            .with_wnr(false)
            .calculate_parity();
        let raw_header: u32 = header.into();
        let mut buffer = [0u8; 12];
        buffer[..4].swap_with_slice(&mut raw_header.to_be_bytes());
        match self.spi.transfer(&mut buffer) {
            Ok(_) => {
                let recv_header_bytes: [u8; 4] = buffer[4..8].try_into().unwrap();
                let recv_header_raw = u32::from_be_bytes(recv_header_bytes);
                let recv_header: ControlCommandHeader = recv_header_raw.into();
                let v: [u8; 4] = buffer[8..].try_into().unwrap();
                Ok(u32::from_be_bytes(v))
            },
            Err(_) => Err(Error::IOError),
        }
    }

    pub fn get_capability(&mut self) -> Result<u32, Error> {
        let mut header = ControlCommandHeader::new()
            .with_length(0)
            .with_address(Registers::Capability as u16)
            .with_memmap(0)
            .with_aid(true)
            .with_wnr(false)
            .calculate_parity();
        let raw_header: u32 = header.into();
        let mut buffer = [0u8; 12];
        buffer[..4].swap_with_slice(&mut raw_header.to_be_bytes());
        match self.spi.transfer(&mut buffer) {
            Ok(_) => {
                let recv_header_bytes: [u8; 4] = buffer[4..8].try_into().unwrap();
                let recv_header_raw = u32::from_be_bytes(recv_header_bytes);
                let recv_header: ControlCommandHeader = recv_header_raw.into();
                if !recv_header.received_header_is_ok() {
                    return Err(Error::HeaderBadError);
                }
                let v: [u8; 4] = buffer[8..].try_into().unwrap();
                Ok(u32::from_be_bytes(v))
            },
            Err(_) => Err(Error::IOError),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IOError,
    HeaderBadError,
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
///
/// It might make sense to move this and other bitfield structs into a separate module
/// to limit irresponsible use of the getters/setters. Then the pub keyword would help us.
#[bitfield(u32)]
struct ControlCommandHeader {
    /// Parity bit (calculated as odd parity)
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
    hdrb: bool,
    __: bool,
}

impl ControlCommandHeader {

    /// Calculate the odd parity for a header, and update the parity bit. Use this instead
    /// of setting the parity via the set_parity method provided by bitfield_struct.
    fn calculate_parity(&mut self) -> Self {
        let raw_header: u32 = (*self).into();
        match raw_header.count_ones() % 2 {
            0 => self.set_parity(true),
            _ => self.set_parity(false),
        }
        return *self;
    }

    /// Check if the received header is ok.
    /// For now this means checking that the hdrb ("header bad") bit is ~not~ set.
    fn received_header_is_ok(&self) -> bool {
        !self.hdrb()
    }
}
