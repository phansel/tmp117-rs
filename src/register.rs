//! The definitions of drivers of the TMP117
#![allow(clippy::identity_op)]
use device_register::{RERegister, RORegister, RWRegister};
use embedded_hal::i2c::ErrorKind;
use modular_bitfield::prelude::*;

/// The address of the register
pub struct Address(pub u8);

/// Temperature register. The value is in 1/7.8125 m°C.
/// Following a reset, the temperature register reads –256 °C until the first conversion,
/// including averaging, is complete. Is in two complements
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RORegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x00)")]
pub struct Temperature(B16);

/// Represent the dataready or alert pin select
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, BitfieldSpecifier)]
#[bits = 1]
pub enum AlertPinSelect {
    ///Alert pin reflects the status of the alert flag
    Alert = 0,

    ///Alert pin reflects the status of the data ready flag
    DataReady = 1,
}

/// Possible polarities
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, BitfieldSpecifier)]
#[bits = 1]
pub enum Polarity {
    ///Polarity set to active low
    ActiveLow = 0,

    ///Polarity set to active high
    ActiveHigh = 1,
}

/// Possible mode selection
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, BitfieldSpecifier)]
#[bits = 1]
pub enum TriggerMode {
    /// Alert mode
    Alert = 0,

    /// Thermal mode
    Thermal = 1,
}

/// Conversion averaging modes. Determines the number of
/// conversion results that are collected and averaged before
/// updating the temperature register. The average is an
/// accumulated average and not a running average.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, BitfieldSpecifier)]
#[bits = 2]
pub enum Average {
    /// No averaging
    NoAverage = 0,

    /// 8 averaged conversions
    Avg8 = 1,

    /// 32 averaged conversions
    Avg32 = 2,

    /// 64 averaged conversions
    Avg64 = 3,
}

/// Conversion cycle. It depends on the average selected. The enum represents the values for no average.
/// | CONV[2:0] | AVG[1:0] = 00 | AVG[1:0] = 01 | AVG[1:0] = 10 | AVG[1:0] = 11 |
/// |-----------|---------------|---------------|---------------|---------------|
/// | 000       | 15.5 ms       | 125 ms        | 500 ms        | 1 s           |
/// | 001       | 125 ms        | 125 ms        | 500 ms        | 1 s           |
/// | 010       | 250 ms        | 250 ms        | 500 ms        | 1 s           |
/// | 011       | 500 ms        | 500 ms        | 500 ms        | 1 s           |
/// | 100       | 1 s           | 1 s           | 1 s           | 1 s           |
/// | 101       | 4s            | 4 is          | 4s            | 4s            |
/// | 110       | 8 s           | 8 S           | 8s            | 8 s           |
/// | 111       | 16 S          | 16 S          | 16 S          | 16 S          |
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, BitfieldSpecifier)]
#[bits = 3]
pub enum Conversion {
    /// 15.5ms cycle time without average.
    Ms15_5 = 0,

    /// 125ms cycle time without average.
    Ms125 = 1,

    /// 250ms cycle time without average.
    Ms250 = 2,

    /// 500ms cycle time without average.
    Ms500 = 3,

    /// 1000ms cycle time without average.
    Ms1000 = 4,

    /// 4000ms cycle time without average.
    Ms4000 = 5,

    /// 8000ms cycle time without average.
    Ms8000 = 6,

    /// 16000ms cycle time without average.
    Ms16000 = 7,
}

/// Conversion mode
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, BitfieldSpecifier)]
#[bits = 2]
pub enum ConversionMode {
    /// Continous conversion mode
    Continuous = 0,

    /// Shutdown conversion mode
    Shutdown = 1,

    /// Oneshot conversion monde
    OneShot = 3,
}

/// Configuration register of the tpm117
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RERegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x01)")]
pub struct Configuration {
    #[skip]
    __: B1,

    /// Software reset. When enabled, cause a reset with a duration of 2ms. The bit will always read back 0
    pub reset: bool,

    /// Data ready or Alert pin select bit.
    pub dr_alert: AlertPinSelect,

    /// Alert pin polarity.
    pub polarity: Polarity,

    /// Thermal/alert mode select
    pub trigger_mode: TriggerMode,

    /// Average used for the conversion
    pub average: Average,

    /// Conversion cycle
    pub conversion: Conversion,

    /// Conversion mode
    pub mode: ConversionMode,

    /// EEPROM busy flag, either caused by programming or power-up
    #[skip(setters)]
    pub eeprom_busy: bool,

    /// Data ready flag.
    /// This flag indicates that the conversion is complete and the
    /// temperature register can be read. Every time the temperature
    /// register or configuration register is read, this bit is cleared. This
    /// bit is set at the end of the conversion when the temperature
    /// register is updated. Data ready can be monitored on the ALERT
    /// pin by setting bit 2 of the configuration register.
    #[skip(setters)]
    pub data_ready: bool,

    /// Alert mode:
    ///   Set when the conversion result is lower than the low limit.
    ///   Cleared when read.
    /// Thermal mode:
    ///   Always 0 in [Thermal](TriggerMode::Thermal) mode.
    #[skip(setters)]
    pub low_alert: bool,

    /// Alert mode:
    ///   Set when the conversion result is higher than the high limit.
    ///   Cleared when read.
    /// Thermal mode:
    ///   Set when the conversion result is higher than the therm limit
    ///   Cleared when the conversion result is lower than the hysteresis
    #[skip(setters)]
    pub high_alert: bool,
}

/// The high limit register is a 16-bit, read/write register that stores the high limit for comparison with the temperature result.
/// One LSB equals 7.8125 m°C. The range of the register is ±256 °C. Negative numbers are represented in binary
/// two's complement format. Following power-up or a general-call reset, the high-limit register is loaded with the
/// stored value from the EEPROM. The factory default reset value is 6000h. Is written in two's complement.
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RWRegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x02)")]
pub struct HighLimit(B16);

/// The low limit register is configured as a 16-bit, read/write register that stores the low limit for comparison with the
/// temperature result. One LSB equals 7.8125 m°C. The range of the register is ±256 °C. Negative numbers
/// are represented in binary two's complement format. The data format is the same as the temperature register.
/// Following power-up or reset, the low-limit register is loaded with the stored value from the EEPROM. The factory
/// default reset value is 8000h.Is written in two's complement.
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RWRegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x03)")]
pub struct LowLimit(B16);

/// The eeprom configuration register
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RERegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x04)")]
pub struct EEPROM {
    #[skip]
    __: B14,

    /// EEPROM busy flag, either caused by programming or power-up
    ///Mirror the `eeprom_busy` in the [Configuration](Configuration) register
    #[skip(setters)]
    pub busy: bool,

    /// If the eeprom is unlock. If unlcoked, any writes to the registers program will be written to the eeprom
    pub unlock: bool,
}

/// The EEPROM1 register is a 16-bit register that be used as a scratch pad by the customer to store general-
/// purpose data. This register has a corresponding EEPROM location. Writes to this address when the EEPROM is
/// locked write data into the register and not to the EEPROM. Writes to this register when the EEPROM is unlocked
/// causes the corresponding EEPROM location to be programmed.
/// To support NIST tracebility, do not delete or reprogram the [UEEPROM1](UEEPROM1) register
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RWRegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x05)")]
pub struct UEEPROM1(B16);

/// Same function as register [UEEPROM1](UEEPROM1) minus the ID for NSIT tracability
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RWRegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x06)")]
pub struct UEEPROM2(B16);

/// Same function as register [UEEPROM1](UEEPROM1) minus the ID for NSIT tracability
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RWRegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x07)")]
pub struct UEEPROM3(B16);

/// This 16-bit register is to be used as a user-defined temperature offset register during system calibration. The
/// offset will be added to the temperature result after linearization. It has a same resolution of 7.8125 m°C and
/// same range of ±256 °C as the temperature result register. The data format is the same as the temperature
/// register. If the added result is out of boundary, then the temperature result will show as the maximum or
/// minimum value. Is written in two's complement.
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RWRegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x08)")]
pub struct TemperatureOffset(B16);

/// Indicates the device ID
#[bitfield]
#[repr(u16)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, RORegister)]
#[register(ty = "Address", err = "ErrorKind", addr = "Address(0x0F)")]
pub struct DeviceID {
    /// Indicates the device ID
    pub device_id: B12,

    /// Indicates the revision number
    pub revision: B4,
}
