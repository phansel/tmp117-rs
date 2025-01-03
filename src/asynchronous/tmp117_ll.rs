//! Async low level driver of the tmp117
use core::marker::PhantomData;

use device_register::Register;
use device_register_async::RegisterInterface;
use embedded_hal::i2c::SevenBitAddress;
use embedded_hal_async::i2c::I2c;

use crate::error::ErrorLL;
use crate::register::Address;

/// Async low level driver of the TPM117. Allows to read, write and edit the registers directly via the i2c bus
pub struct Tmp117LL<T, E> {
    i2c: T,
    addr: u8,
    e: PhantomData<E>,
}

impl<T, E> Tmp117LL<T, E>
where
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
{
    /// Creates a new instace of the Tmp117 from an i2c bus
    pub fn new(i2c: T, addr: u8) -> Self {
        Self {
            i2c,
            addr,
            e: PhantomData,
        }
    }
}

impl<T, E, R> RegisterInterface<R, Address> for Tmp117LL<T, E>
where
    R: Register<Address = Address> + Clone + TryFrom<u16>,
    u16: From<R>,
    T: I2c<SevenBitAddress, Error = E>,
    E: embedded_hal::i2c::Error,
{
    type Error = ErrorLL<E>;

    async fn read_register(&mut self) -> Result<R, Self::Error> {
        let mut buff = [0; 2];
        self.i2c
            .write_read(self.addr, &[R::ADDRESS.0], &mut buff)
            .await
            .map_err(ErrorLL::Bus)?;
        let val = u16::from_be_bytes(buff[0..2].try_into().unwrap());
        R::try_from(val).map_err(|_| ErrorLL::InvalidData)
    }

    async fn write_register(&mut self, register: &R) -> Result<(), Self::Error> {
        let val: u16 = register.clone().into();
        let packet = val.to_be_bytes();

        self.i2c
            .write(self.addr, &[R::ADDRESS.0, packet[0], packet[1]])
            .await
            .map_err(ErrorLL::Bus)
    }
}
