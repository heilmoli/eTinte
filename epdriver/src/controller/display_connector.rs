use embedded_hal::blocking::spi::{Write, Transfer};
use embedded_hal::digital::v2::{OutputPin, InputPin};
use embedded_hal::blocking::delay::DelayMs;


use core::result;

const TMP_BUFFER_SIZE: usize = 320 * 384;

#[derive(Debug)]
pub enum Error {
    SpiWriteError,
    DelayError,
    BusyPinReadError,
    // BusyPinWriteError
    // PinReadError,
    ResetPinWriteError,
    DcPinWriteError
}

pub type Result<T> = result::Result<T, Error>;

pub trait DisplayConnector {
    fn reset(&mut self) -> Result<()>;
    fn is_busy(&self) -> Result<bool>;
    fn send_command(&mut self, command: u8) -> Result<()>;
    fn send_data_with<F>(&mut self, repeats: u32, source: F) -> Result<()> where F: Fn(u32) -> u8;
    fn send_data(&mut self, data: &[u8]) -> Result<()>;
    fn delay_ms(&mut self, ms: u16) -> Result<()>;
}

//cat /sys/module/spidev/parameters/bufsiz
pub struct SpiConnector<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    spi: SPI,
    rst: OUT,
    dc: OUT,
    busy: IN,
    delay: DELAY,
    chunk_size: usize,
    tmp_buffer: Option<[u8; TMP_BUFFER_SIZE]>,
}


impl<SPI, OUT, IN, DELAY> SpiConnector<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    pub(crate) fn new(spi: SPI, rst: OUT, dc: OUT, busy: IN, delay: DELAY, chunk_size: usize) -> SpiConnector<SPI, OUT, IN, DELAY> {
        SpiConnector {
            spi,
            rst,
            dc,
            busy,
            delay,
            chunk_size, // depends on systems
            tmp_buffer: Option::None,
        }
    }


    fn write(&mut self, data: &[u8]) -> Result<()> {
        for data_chunk in data.chunks(self.chunk_size) {
            self.spi.write(data_chunk).map_err(|_e| Error::SpiWriteError)?;
        };
        Result::Ok(())
    }
}

impl<SPI, OUT, IN, DELAY> DisplayConnector for SpiConnector<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    fn reset(&mut self) -> Result<()> {
        self.rst.set_high().map_err(|_e| Error::ResetPinWriteError)?;
        self.delay.delay_ms(200);
        self.rst.set_low().map_err(|_e| Error::ResetPinWriteError)?;
        self.delay.delay_ms(4);
        let r = self.rst.set_high().map_err(|_e| Error::ResetPinWriteError);
        self.delay.delay_ms(200);
        r
    }

    fn is_busy(&self) -> Result<bool> {
        self.busy.is_low().map_err(|_e| Error::BusyPinReadError)
    }

    fn send_command(&mut self, command: u8) -> Result<()> {
        self.dc.set_low().map_err(|_e| Error::DcPinWriteError)?;
        self.write(&[command])?;
        self.delay.delay_ms(4);
        Ok(())
    }

    fn send_data_with<F>(&mut self, repeats: u32, source: F) -> Result<()> where F: Fn(u32) -> u8 {
        let mut buffer = self.tmp_buffer.take().unwrap_or_else(|| {
            [0; TMP_BUFFER_SIZE]
        });

        let buffer_size = buffer.len();
        let mut i = 0;
        for x in 0..repeats {
            {
                buffer[i] = source(x);
            }
            i += 1;
            if i % buffer_size == 0 {
                self.send_data(&buffer)?;
                i = 0;
            }
        }
        if i > 0 {
            self.send_data(&buffer[0..i])?;
        }
        self.delay.delay_ms(4);
        //  self.send_command(0x11);
        self.tmp_buffer = Some(buffer);
        Result::Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> Result<()> {
        self.dc.set_high().map_err(|_e| Error::DcPinWriteError)?;
        self.write(data)?;
        Ok(())
    }

    fn delay_ms(&mut self, ms: u16) -> Result<()> {
        self.delay.delay_ms(ms);
        Ok(())
    }
}
