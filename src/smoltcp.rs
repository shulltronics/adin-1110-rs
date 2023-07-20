//! A module for implementing the `smoltcp` Device trait for this driver

#![cfg(feature = "smoltcp")]

use crate::ADIN1110;
use smoltcp::phy::Device;

impl Device for ADIN1110 {

}
