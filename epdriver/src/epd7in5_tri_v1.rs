use crate::controller::il0371::*;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Write, Transfer};
use embedded_hal::digital::v2::{OutputPin, InputPin};
use crate::controller::display_connector::SpiConnector;

use crate::controller::display_connector::Result;
use crate::controller::gd7965::GD7965;
use crate::display::EPaperDisplay;

pub struct EPaper75TriColour<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    controller: IL0371<SpiConnector<SPI, OUT, IN, DELAY>>,
    pub width: u16,
    pub height: u16,
}

impl<SPI, OUT, IN, DELAY> EPaper75TriColour<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    pub fn new(spi: SPI, rst: OUT, dc: OUT, busy: IN, delay: DELAY, chunk_size: usize) -> EPaper75TriColour<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16>{

        let connector = SpiConnector::new(spi, rst, dc, busy, delay, chunk_size); // TODO: set buffer size here
        let controller = IL0371::new(connector);
        EPaper75TriColour { controller, width: 640, height: 384 }
    }

    pub fn sleep(&mut self) -> Result<()> {
        self.controller.pof_power_off()?;
        self.controller.await_ready_state()?;
        self.controller.dslp_deep_sleep()
    }

    pub fn clear_with_val(&mut self, val: u8) -> Result<()>{
        let size: u32 = (self.width as u32 * self.height as u32) / 2;
        self.controller.transmit_with(size, |_| val)?;
        self.controller.pon_power_on()?;
        self.controller.await_ready_state()?;
        self.controller.drf_display_refresh()?;
        self.controller.await_ready_state()?;
        self.controller.pof_power_off()
    }

    fn map_pix_value(val: u8) -> u8 {
        match val {
            0 => 0x0,
            1 => 0x4,
            _ => 0x3
        }
    }
}

impl<SPI, OUT, IN, DELAY> EPaperDisplay for EPaper75TriColour<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {

    fn init(&mut self) -> Result<()> {
        self.controller.reset()?;
        self.controller.pwr_power_setting(PWRFlags::EDATA_SEL | PWRFlags::EDATA_SET | PWRFlags::VSOURCE_LV_EN | PWRFlags::VSOURCE_EN | PWRFlags::VGATE_EN)?;
        self.controller.psr_panel_setting(PSRFlags::RES_600_448 | PSRFlags::UD | PSRFlags::SHL | PSRFlags::SHD_N | PSRFlags::RST_N | PSRFlags::MYSTERY)?;
        self.controller.btst_booster_soft_start(0xc7, 0xcc, 0x28)?;
        self.controller.pon_power_on()?;
        self.controller.await_ready_state()?;
        self.controller.pll_control(0x3c)?;
        self.controller.tse_temperature_sensor_calibration(false, 0)?;
        self.controller.cdi_vcom_and_data_interval_settings(3, true, 7)?;
        self.controller.tcon_setting(0x22)?;
        self.controller.tres_resolution(self.width, self.height)?;
        self.controller.vcom_dc_setting(0x1E)?;
     //   self.controller.dam_spi_flash_control(false)?;
        self.controller.define_flash(3)
    }

    fn clear(&mut self) -> Result<()>{
        let size: u32 = (self.width as u32 * self.height as u32) / 2;
        self.controller.transmit_with(size, |_| 0x00)?;
        self.controller.pon_power_on()?;
        self.controller.await_ready_state()?;
        self.controller.drf_display_refresh()?;
        self.controller.await_ready_state()?;
        self.controller.pof_power_off()
    }

    fn push_image_with<F>(&mut self, source: F) -> Result<()> where F: Fn(u32, u32) -> u8 {
        let linebytes : u32 = (self.width / 2) as u32;
        let size: u32 = (linebytes * self.height as u32);
        self.controller.transmit_with(size, |offset| {
            let y = offset / linebytes;
            let x = (offset % linebytes) * 2;
            let p1 = Self::map_pix_value(source(x, y)) << 4;
            let p2 = Self::map_pix_value(source(x + 1, y));
            p1 | p2
        })?;


        self.controller.pon_power_on()?;
        self.controller.await_ready_state()?;
        self.controller.drf_display_refresh()?;
        self.controller.await_ready_state()?;
        self.controller.pof_power_off()
        // think this fixes high contrast situations like lines
        // not sure it works or really adds anythign
        //    self.controller.ipc_image_process(true,3);
        // self.controller.drf_display_refresh();
        // self.controller.await_ready_state();
        // self.controller.dam_spi_flash_control(true);
        // self.controller.flash_data();
        // self.controller.dam_spi_flash_control(false);
        // self.controller.pof_power_off();
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }
}


#[cfg(test)]
mod tests {
    use embedded_hal::blocking::spi::{Write, Transfer};
    use embedded_hal::blocking::delay::DelayMs;
    use embedded_hal::digital::v2::{ InputPin, OutputPin };
    use crate::epd7in5_tri_v1::EPaper75TriColour;
    use crate::display::EPaperDisplay;

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

    #[test]
    fn test_init() {
        let mut display = EPaper75TriColour::new(
            MockSpi {},
            MockPin { name: "rst", state: true },
            MockPin { name: "dc", state: true },
            MockPin { name: "busy", state: true },
            MockDelay {},
            4096);
        display.init();
    }

    #[test]
    fn test_push_image_with() {
        let mut display = EPaper75TriColour::new(
            MockSpi {},
            MockPin { name: "rst", state: true },
            MockPin { name: "dc", state: true },
            MockPin { name: "busy", state: true },
            MockDelay {},
            4096);
        display.push_image_with(|x,y| {
            let xr = x as i32 - 100;
            let yr= y as i32 - 100;
            return 0; //if xr*xr+yr*yr > 100 { 1 } else { 2 };
        });
    }

    #[test]
    fn test_nothing() {

    }

}