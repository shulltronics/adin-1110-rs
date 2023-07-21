#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{
    prelude::*,
    peripherals::Peripherals,
    clock::ClockControl,
    timer::TimerGroup,
    Rtc,
    Delay,
    gpio::IO,
    spi::{Spi, SpiMode},
};

use adin1110::ADIN1110;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    
    let mut status_led = io.pins.gpio2.into_push_pull_output();

    let sclk = io.pins.gpio18;
    let miso = io.pins.gpio19;
    let mosi = io.pins.gpio23;
    let cs = io.pins.gpio5;

    let mut spi = Spi::new(
        peripherals.SPI2,
        sclk,
        mosi,
        miso,
        cs,
        100u32.kHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut delay = Delay::new(&clocks);

    println!("Welcome to the ADIN1110-rs testing app!");

    let mut adin1110 = ADIN1110::new(spi);

    let mut counter = 0;
    loop {
        if (counter % 2) == 0 {
            status_led.set_high().unwrap();
        } else {
            status_led.set_low().unwrap();
        }
        counter += 1;

        match adin1110.get_idver() {
            Ok(id) => println!("Id is: {:#8x?}", id),
            Err(e) => println!("error in get_idver!"),
        }

        match adin1110.get_phyid() {
            Ok(n) => println!("phyid is: {:#8x?}", n),
            Err(e) => println!("Error! {:?}", e),
        }

        match adin1110.get_capability() {
            Ok(n) => println!("capability reg is: {:#08x?}", n),
            Err(e) => println!("error! {:?}", e),
        }

        delay.delay_ms(500u32);
    }
}
