#![no_std]
#![no_main]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{interrupt, twim::Twim};
use tmp117::{register::Average, ContinuousMode, OneShotMode, ShutdownMode, Tmp117};
use {defmt_rtt as _, embassy_nrf as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Start");

    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let twi = Twim::new(p.TWISPI0, irq, p.P1_10, p.P1_11, Default::default());

    let tmp = Tmp117::<0x49, _, _, _>::new(twi);

    // Read and goes to shutdown mode
    info!("Transition to one shot");
    let tmp: Tmp117<0x49, _, _, OneShotMode> = tmp.to_oneshot(Average::NoAverage).unwrap();

    info!("Reading temp");
    let (temperature, tmp) = tmp.wait_temp().unwrap();

    // Verify type
    let tmp: Tmp117<0x49, _, _, ShutdownMode> = tmp;
    info!("Temperature {}", temperature);

    info!("To continuous");
    let mut tmp: Tmp117<0x49, _, _, ContinuousMode> =
        tmp.to_continuous(Default::default()).unwrap();

    for _ in 0..10 {
        let temp = tmp.wait_temp().unwrap();
        info!("Temperature {}", temp);
    }

    let mut tmp: Tmp117<0x49, _, _, ShutdownMode> = tmp.to_shutdown().unwrap();

    let mut eeprom_data = tmp.read_eeprom().unwrap();
    info!("Eeprom data before: {}", eeprom_data);

    eeprom_data[2] += 1;

    info!("Writing {} to eeprom", eeprom_data);
    tmp.write_eeprom(eeprom_data).unwrap();

    let eeprom_data2 = tmp.read_eeprom().unwrap();
    assert_eq!(eeprom_data, eeprom_data2);

    cortex_m::asm::bkpt();
}
