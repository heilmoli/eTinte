use crate::controller::il0371::*;

use crate::controller::display_connector::{DisplayConnector, Result};

use crate::controller::gd7965::GD7965;
use crate::display::EPaperDisplay;

pub struct EPaper75TriColour<T : DisplayConnector> {
    controller: IL0371<T>,
    pub width: u16,
    pub height: u16,
}

impl<T: DisplayConnector> EPaper75TriColour<T>  {
    pub fn new(connector : T) -> EPaper75TriColour<T> {
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

impl<T : DisplayConnector> EPaperDisplay for EPaper75TriColour<T> {

    fn init(&mut self) -> Result<()> {
        self.controller.reset()?;
        self.controller.pwr_power_setting(PWRFlags::EDATA_SEL | PWRFlags::EDATA_SET | PWRFlags::VSOURCE_LV_EN | PWRFlags::VSOURCE_EN | PWRFlags::VGATE_EN)?;
        self.controller.psr_panel_setting(PSRFlags::RES_600_448 | PSRFlags::UD | PSRFlags::SHL | PSRFlags::SHD_N | PSRFlags::RST_N | PSRFlags::MYSTERY)?;
        self.controller.pll_control(0x3c)?;
        self.controller.vcom_dc_setting(0x1E)?;
        self.controller.btst_booster_soft_start(0xc7, 0xcc, 0x28)?;
        self.controller.cdi_vcom_and_data_interval_settings(3, true, 7)?;
        self.controller.tcon_setting(0x22)?;
        self.controller.dam_spi_flash_control(false)?;
        self.controller.tres_resolution(self.width, self.height)?;
        self.controller.define_flash(3)

        // self.controller.pon_power_on()?;
        // self.controller.await_ready_state()?;
        // self.controller.tse_temperature_sensor_calibration(false, 0)?;

    }

    fn clear(&mut self) -> Result<()>{
        let size: u32 = (self.width as u32 * self.height as u32) / 2;
        self.controller.transmit_with(size, |_| 0x00)?;
        self.controller.pon_power_on()?;
        self.controller.await_ready_state()?;
        self.controller.drf_display_refresh()?;
        self.controller.await_ready_state()
//        self.controller.pof_power_off()
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