//! Async low level driver of the tmp117
use embedded_hal::i2c::SevenBitAddress;
use embedded_hal_async::i2c::I2c;

use crate::register::{EditableRegister, Register, WritableRegister};

/// Async low level driver of the TPM117. Allows to read, write and edit the registers directly via the i2c bus
pub struct Tmp117LL<const ADDR: u8, T, E>
where
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
{
    i2c: T,
}

impl<const ADDR: u8, T, E> Tmp117LL<ADDR, T, E>
where
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
{
    /// Creates a new instace of the Tmp117 from an i2c bus
    pub fn new(i2c: T) -> Self {
        Self { i2c }
    }

    async fn write_internal<R>(&mut self, reg: R) -> Result<(), E>
    where
        R: Register,
        u16: From<R>,
    {
        let val: u16 = reg.into();
        let packet = val.to_be_bytes();

        self.i2c
            .write(ADDR, &[R::ADDRESS, packet[0], packet[1]])
            .await
    }

    /// Read a register value
    pub async fn read<R>(&mut self) -> Result<R, E>
    where
        R: Register + From<u16>,
    {
        let mut buff = [0; 2];
        self.i2c.write(ADDR, &[R::ADDRESS]).await?;
        self.i2c.read(ADDR, &mut buff).await?;
        let val = u16::from_be_bytes(buff[0..2].try_into().unwrap());
        Ok(val.into())
    }

    /// Edit a register, use the argument from the passed function and edit the fields,
    /// do not overwrite the register entirely, some bits are reserved
    pub async fn edit<R, F>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(R) -> R,
        R: EditableRegister + From<u16>,
        u16: From<R>,
    {
        let val: R = self.read().await?;
        let new_val = f(val);
        self.write_internal(new_val).await
    }

    /// Write to a register
    pub async fn write<R>(&mut self, reg: R) -> Result<(), E>
    where
        R: WritableRegister,
        u16: From<R>,
    {
        self.write_internal(reg).await
    }
}