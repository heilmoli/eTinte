use crate::controller::il0371::*;
use crate::controller::display_connector::Result;

#[derive(Debug)]
pub enum DisplayError {

}

pub trait EPaperDisplay {
    fn init(&mut self) -> Result<()>;
    fn push_image_with<F>(&mut self, source: F) -> Result<()> where F: Fn(u32, u32) -> u8;
    fn clear(&mut self) -> Result<()>;
    fn width(&self) -> u16;
    fn height(&self) -> u16;

}
#[cfg(test)]
mod tests {
    use embedded_hal::blocking::spi::{Write, Transfer};
    use embedded_hal::blocking::delay::DelayMs;
    use embedded_hal::digital::v2::{ InputPin, OutputPin };

    struct MockPin {
        name: &'static str,
        state: bool,
    }

    impl OutputPin for MockPin {
        type Error = ();

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.state = false;
            Result::Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.state = true;
            Result::Ok(())
        }
    }

    impl InputPin for MockPin {
        type Error = ();

        fn is_high(&self) -> Result<bool, Self::Error> {
            Result::Ok(self.state)
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Result::Ok(!self.state)
        }
    }


    struct MockSpi {}

    impl Write<u8> for MockSpi {
        type Error = ();

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            Result::Ok(())
        }
    }

    impl Transfer<u8> for MockSpi {
        type Error = ();

        fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
            Result::Ok(&[])
        }
    }

    struct MockDelay {}

    impl DelayMs<u16> for MockDelay {
        fn delay_ms(&mut self, ms: u16) {}
    }
}


