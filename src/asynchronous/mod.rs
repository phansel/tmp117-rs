//! Async drivers of the tmp117

use core::marker::PhantomData;

use device_register_async::{EditRegister, ReadRegister, WriteRegister};
use embedded_hal::i2c::SevenBitAddress;
use embedded_hal_async::{digital::Wait, i2c::I2c};

use crate::{
    register::*, Alert, ContinousConfig, ContinuousMode, Error, OneShotMode, ShutdownMode,
    UnknownMode, CELCIUS_CONVERSION,
};

use self::tmp117_ll::Tmp117LL;
pub mod tmp117_ll;

/// The status of the alert pin
enum AlertPin<P> {
    /// Unkown, right after boot
    Unkown(P),
    /// Currently in data ready
    DataReady(P),
    /// Currently in alert
    Alert(P),
}
impl<P> AlertPin<P> {
    /// Borrow a mutable reference to then internal pin without caring for it's state
    pub fn borrow_mut(&mut self) -> &mut P {
        match self {
            AlertPin::Unkown(p) => p,
            AlertPin::DataReady(p) => p,
            AlertPin::Alert(p) => p,
        }
    }
}

/// The TMP117 driver. Note that the alert pin is optional, but it is recommended to pass it if possible
/// If the alert pin is `None`, the driver will poll the config register instead of waiting for the pin.
pub struct Tmp117<const ADDR: u8, T, E, P, M>
where
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
    P: Wait,
{
    tmp_ll: Tmp117LL<ADDR, T, E>,
    alert: Option<AlertPin<P>>,
    mode: PhantomData<M>,
}

impl<const ADDR: u8, T, E, P, M> Tmp117<ADDR, T, E, P, M>
where
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
    P: Wait,
{
    /// Create a new tmp117 from a i2c bus
    pub fn new(i2c: T, alert: Option<P>) -> Tmp117<ADDR, T, E, P, UnknownMode> {
        Tmp117::<ADDR, T, E, P, UnknownMode> {
            tmp_ll: Tmp117LL::new(i2c),
            alert: alert.map(|p| AlertPin::Unkown(p)),
            mode: PhantomData,
        }
    }

    /// Create a new tmp117 from a low level tmp117 driver
    pub fn new_from_ll(
        tmp_ll: Tmp117LL<ADDR, T, E>,
        alert: Option<P>,
    ) -> Tmp117<ADDR, T, E, P, UnknownMode> {
        Tmp117::<ADDR, T, E, P, UnknownMode> {
            tmp_ll,
            alert: alert.map(|p| AlertPin::Unkown(p)),
            mode: PhantomData,
        }
    }

    async fn wait_eeprom(&mut self) -> Result<(), Error> {
        let mut configuration: Configuration = self.tmp_ll.read().await.map_err(Error::Bus)?;
        while configuration.eeprom_busy() {
            configuration = self.tmp_ll.read().await.map_err(Error::Bus)?;
        }

        Ok(())
    }

    /// Go to continuous mode
    pub async fn to_continuous(
        mut self,
        config: ContinousConfig,
    ) -> Result<Tmp117<ADDR, T, E, P, ContinuousMode>, Error> {
        self.tmp_ll
            .edit(|mut r: Configuration| {
                r.set_mode(ConversionMode::Continuous);
                if let Some(val) = config.average {
                    r.set_average(val);
                }
                if let Some(val) = config.conversion {
                    r.set_conversion(val);
                }
                r
            })
            .await
            .map_err(Error::Bus)?;
        if let Some(val) = config.high {
            let high: HighLimit = ((val / CELCIUS_CONVERSION) as u16).into();
            self.tmp_ll.write(high).await.map_err(Error::Bus)?;
        }
        if let Some(val) = config.low {
            let low: LowLimit = ((val / CELCIUS_CONVERSION) as u16).into();
            self.tmp_ll.write(low).await.map_err(Error::Bus)?;
        }
        if let Some(val) = config.offset {
            let off: TemperatureOffset = ((val / CELCIUS_CONVERSION) as u16).into();
            self.tmp_ll.write(off).await.map_err(Error::Bus)?;
        }

        Ok(Tmp117::<ADDR, T, E, P, ContinuousMode> {
            tmp_ll: self.tmp_ll,
            alert: self.alert,
            mode: PhantomData,
        })
    }

    /// Go to oneshot mode
    pub async fn to_oneshot(
        mut self,
        average: Average,
    ) -> Result<Tmp117<ADDR, T, E, P, OneShotMode>, Error> {
        self.tmp_ll
            .edit(|r: Configuration| r.with_mode(ConversionMode::OneShot).with_average(average))
            .await
            .map_err(Error::Bus)?;

        Ok(Tmp117::<ADDR, T, E, P, OneShotMode> {
            tmp_ll: self.tmp_ll,
            alert: self.alert,
            mode: PhantomData,
        })
    }

    /// Go to shotdown mode
    pub async fn to_shutdown(mut self) -> Result<Tmp117<ADDR, T, E, P, ShutdownMode>, Error> {
        self.tmp_ll
            .edit(|r: Configuration| r.with_mode(ConversionMode::Shutdown))
            .await
            .map_err(Error::Bus)?;

        Ok(Tmp117::<ADDR, T, E, P, ShutdownMode> {
            tmp_ll: self.tmp_ll,
            alert: self.alert,
            mode: PhantomData,
        })
    }

    /// Reset  the device
    pub async fn reset(mut self) -> Result<Tmp117<ADDR, T, E, P, UnknownMode>, Error> {
        self.tmp_ll
            .edit(|r: Configuration| r.with_reset(true))
            .await
            .map_err(Error::Bus)?;

        Ok(Tmp117::<ADDR, T, E, P, UnknownMode> {
            tmp_ll: self.tmp_ll,
            alert: self.alert,
            mode: PhantomData,
        })
    }

    /// Write data to user eeprom. Note that this is blocking because we wait for write on the eeprom to complete
    pub async fn write_eeprom(&mut self, values: [u16; 3]) -> Result<(), Error> {
        self.wait_eeprom().await?;
        self.tmp_ll
            .write(UEEPROM1::from(values[0]))
            .await
            .map_err(Error::Bus)?;

        self.wait_eeprom().await?;
        self.tmp_ll
            .write(UEEPROM2::from(values[1]))
            .await
            .map_err(Error::Bus)?;

        self.wait_eeprom().await?;
        self.tmp_ll
            .write(UEEPROM3::from(values[2]))
            .await
            .map_err(Error::Bus)?;

        Ok(())
    }

    /// Read the data from the eeprom
    pub async fn read_eeprom(&mut self) -> Result<[u16; 3], Error> {
        let u1: UEEPROM1 = self.tmp_ll.read().await.map_err(Error::Bus)?;
        let u2: UEEPROM2 = self.tmp_ll.read().await.map_err(Error::Bus)?;
        let u3: UEEPROM3 = self.tmp_ll.read().await.map_err(Error::Bus)?;

        Ok([u1.into(), u2.into(), u3.into()])
    }
}

impl<const ADDR: u8, T, E, P> Tmp117<ADDR, T, E, P, OneShotMode>
where
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
    P: Wait,
{
    /// Read the temperature and goes to shutdown mode since it's a oneshot
    pub async fn read_temp(mut self) -> Result<(f32, Tmp117<ADDR, T, E, P, ShutdownMode>), Error> {
        let config: Configuration = self.tmp_ll.read().await.map_err(Error::Bus)?;
        if !config.data_ready() {
            return Err(Error::DataNotReady);
        }

        let temp: Temperature = self.tmp_ll.read().await.map_err(Error::Bus)?;
        // Convert to i16 for two complements
        let val = (u16::from(temp) as i16) as f32 * CELCIUS_CONVERSION;
        Ok((
            val,
            Tmp117::<ADDR, T, E, P, ShutdownMode> {
                tmp_ll: self.tmp_ll,
                alert: self.alert,
                mode: PhantomData,
            },
        ))
    }
}

impl<const ADDR: u8, T, E, P> Tmp117<ADDR, T, E, P, ContinuousMode>
where
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
    P: Wait,
{
    async fn read_temp_raw(&mut self) -> Result<f32, Error> {
        let temp: Temperature = self.tmp_ll.read().await.map_err(Error::Bus)?;

        // Convert to i16 for two complements
        let val = (u16::from(temp) as i16) as f32 * CELCIUS_CONVERSION;
        Ok(val)
    }

    /// Read the temperature
    pub async fn read_temp(&mut self) -> Result<f32, Error> {
        let config: Configuration = self.tmp_ll.read().await.map_err(Error::Bus)?;
        if !config.data_ready() {
            return Err(Error::DataNotReady);
        }

        self.read_temp_raw().await
    }

    /// Wait for the data to be ready and read the temperature after
    pub async fn wait_read_temp(&mut self) -> Result<f32, Error> {
        if let Some(p) = &mut self.alert {
            if let AlertPin::DataReady(_) = p {
            } else {
                self.tmp_ll
                    .edit(|r: Configuration| {
                        r.with_dr_alert(AlertPinSelect::DataReady)
                            .with_polarity(Polarity::ActiveHigh)
                    })
                    .await
                    .map_err(Error::Bus)?;
            }
            p.borrow_mut()
                .wait_for_high()
                .await
                .map_err(|_| Error::AlertPin)?;
            self.alert.as_ref().map(|v| Some(AlertPin::DataReady(v)));
            self.read_temp_raw().await
        } else {
            loop {
                let res = self.read_temp().await;
                if let Err(Error::DataNotReady) = res {
                    continue;
                } else {
                    return res;
                }
            }
        }
    }

    /// Check if an alert was triggered since the last calll
    pub async fn check_alert(&mut self) -> Result<Alert, Error> {
        let config: Configuration = self.tmp_ll.read().await.map_err(Error::Bus)?;
        if config.high_alert() && config.low_alert() {
            Ok(Alert::HighLow)
        } else if config.high_alert() {
            Ok(Alert::High)
        } else if config.low_alert() {
            Ok(Alert::Low)
        } else {
            Ok(Alert::None)
        }
    }

    /// Wait for an alert to come and return it's value
    pub async fn wait_alert(&mut self) -> Result<Alert, Error> {
        if let Some(p) = &mut self.alert {
            if let AlertPin::Alert(_) = p {
            } else {
                self.tmp_ll
                    .edit(|r: Configuration| {
                        r.with_dr_alert(AlertPinSelect::Alert)
                            .with_polarity(Polarity::ActiveHigh)
                    })
                    .await
                    .map_err(Error::Bus)?;
            }
            p.borrow_mut()
                .wait_for_high()
                .await
                .map_err(|_| Error::AlertPin)?;
            self.alert.as_ref().map(|v| Some(AlertPin::Alert(v)));
            self.check_alert().await
        } else {
            loop {
                let alert = self.check_alert().await;
                if let Ok(Alert::None) = alert {
                    continue;
                } else {
                    return alert;
                }
            }
        }
    }
}
