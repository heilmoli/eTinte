use crate::controller::il0371::*;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Write, Transfer};
use embedded_hal::digital::v2::{OutputPin, InputPin};
use crate::controller::display_connector::SpiConnector;

use crate::controller::display_connector::Result;
use crate::controller::gd7965::{ GD7965, PWRFlags, PSRFlags };
use crate::display::EPaperDisplay;


pub struct EPaper75TriColourV2<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    controller: GD7965<SpiConnector<SPI, OUT, IN, DELAY>>,
    pub width: u16,
    pub height: u16,
}

impl<SPI, OUT, IN, DELAY> EPaper75TriColourV2<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    pub fn new(spi: SPI, rst: OUT, dc: OUT, busy: IN, delay: DELAY, chunk_size: usize) -> EPaper75TriColourV2<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16>{

        let connector = SpiConnector::new(spi, rst, dc, busy, delay, chunk_size); // TODO: set buffer size here
        let controller = GD7965::new(connector);
        EPaper75TriColourV2 { controller, width: 800, height: 480 }
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
}

impl<SPI, OUT, IN, DELAY> EPaperDisplay for EPaper75TriColourV2<SPI, OUT, IN, DELAY> where SPI: Write<u8> + Transfer<u8>, OUT: OutputPin, IN: InputPin, DELAY: DelayMs<u16> {
    fn init(&mut self) -> Result<()> {
        self.controller.reset()?;
        self.controller.pwr_power_setting(PWRFlags::VSR_EN | PWRFlags::VS_EN | PWRFlags::VG_EN | PWRFlags::VG_LVL_20V, 15.0, -15.0, 3.0)?;
        self.controller.pon_power_on()?;
        self.controller.await_ready_state()?;
        self.controller.psr_panel_setting(PSRFlags::UD | PSRFlags::SHL | PSRFlags::SHD_N | PSRFlags::RST_N )?;
        self.controller.tres_resolution(800, 480)?;
        self.controller.duspi_dual_spi_mode(false, false)?;
        self.controller.cdi_vcom_and_data_interval_settings(false, 1, false, 1, 7)?;
        self.controller.tcon_setting(2,2)?;
        self.controller.gss_gate_source_start_setting(0,0)
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
        let bytes_per_line = (self.width / 8) as u32;
        let size: u32 = (self.width as u32 * self.height as u32);
        self.controller.transmit_with(size, |offset| {
            let y = offset / bytes_per_line;
            let x = (offset % bytes_per_line) * 8;
            let p7 = ((source(x, y) == 2) as u8) << 7;
            let p6 = ((source(x+1, y) == 2) as u8) << 6;
            let p5 = ((source(x+2, y) ==  2) as u8) << 5;
            let p4 = ((source(x+3, y) == 2) as u8) << 4;
            let p3 = ((source(x+4, y) ==  2) as u8) << 3;
            let p2 = ((source(x+5, y) == 2) as u8) << 2;
            let p1 = ((source(x+6, y) == 2) as u8)  <<  1;
            let p0 = ((source(x+7, y) == 2) as u8);
            p7 | p6 | p5 | p4 | p3 | p2 | p1 | p0
        })?;
        self.controller.await_ready_state()?;
        self.controller.transmit_with2(size, |offset| {
            let y = offset / bytes_per_line;
            let x = (offset % bytes_per_line) * 8;
            let p7 = ((source(x, y) == 1) as u8) << 7;
            let p6 = ((source(x+1, y) == 1) as u8) << 6;
            let p5 = ((source(x+2, y) == 1) as u8) << 5;
            let p4 = ((source(x+3, y) == 1) as u8) << 4;
            let p3 = ((source(x+4, y) == 1) as u8) << 3;
            let p2 = ((source(x+5, y) == 1) as u8) << 2;
            let p1 = ((source(x+6, y) == 1) as u8) << 1;
            let p0 = (source(x+7, y) == 1) as u8;
            p7 | p6 | p5 | p4 | p3 | p2 | p1 | p0
        })?;



        // self.controller.pon_power_on()?;
        // self.controller.await_ready_state()?;
         self.controller.drf_display_refresh();
        self.controller.await_ready_state()

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
