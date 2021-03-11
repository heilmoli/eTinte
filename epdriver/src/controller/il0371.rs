use crate::controller::display_connector::{DisplayConnector,Result};


/*
// http://www.e-paper-display.com/download_detail/downloadsId=536.html
// https://v4.cecdn.yun300.cn/100001_1909185148/IL0371.pdf
// seems to be same as UC8159C (https://www.buydisplay.com/download/ic/UC8159C.pdf)

 */
// driver

bitflags! {
    pub struct PSRFlags: u16 {
        const RES_640_480 = 0b0000_0000_0000_0000;
        const RES_600_450 = 0b0100_0000_0000_0000;
        const RES_640_448 = 0b1000_0000_0000_0000;
        const RES_600_448 = 0b1100_0000_0000_0000;

        const LUT_EN =      0b0010_0000_0000_0000;
        const UD =          0b0000_1000_0000_0000;
        const SHL =         0b0000_0100_0000_0000;
        const SHD_N =       0b0000_0010_0000_0000;
        const RST_N =       0b0000_0001_0000_0000;
        const VCM_HZ =      0b0000_0000_0001_0000;
        const MYSTERY =     0b0000_0000_0000_1000; // this gets used in the sample code but is undocumented
    }
}

bitflags! {
    pub struct PWRFlags: u16 {
        const EDATA_SEL     = 0b0010_0000_0000_0000;
        const EDATA_SET     = 0b0001_0000_0000_0000;
        const VSOURCE_LV_EN = 0b0000_0100_0000_0000;
        const VSOURCE_EN    = 0b0000_0010_0000_0000;
        const VGATE_EN      = 0b0000_0001_0000_0000;

        const VGHL_LVL_20V = 0b0000_0000_0000_0000;
        const VGHL_LVL_19V = 0b0000_0000_0000_0001;
        const VGHL_LVL_18V = 0b0000_0000_0000_0010;
        const VGHL_LVL_17V = 0b0000_0000_0000_0011;
    }
}
bitflags! {
    pub struct PFSFlags: u8 {
        const T_VDS_OFF_1FRAME = 0b0000_0000;
        const T_VDS_OFF_2FRAME = 0b0000_1000;
        const T_VDS_OFF_3FRAME = 0b0001_0000;
        const T_VDS_OFF_4FRAME = 0b0001_1000;
    }
}


pub struct IL0371<T> where T: DisplayConnector {
    connector: T
}


impl<T> IL0371<T> where T: DisplayConnector {
    pub fn new(connector: T) -> IL0371<T> {
        IL0371 {
            connector
        }
    }

    #[allow(dead_code)]
    pub(crate) fn reset(&mut self) -> Result<()> {
        self.connector.reset()
    }

    #[allow(dead_code)]
    pub(crate) fn psr_panel_setting(&mut self, psr_flags: PSRFlags) -> Result<()> {
        self.connector.send_command(0)?;
        self.connector.send_data(&psr_flags.bits.to_be_bytes())
    }

    #[allow(dead_code)]
    pub(crate) fn pwr_power_setting(&mut self, pwr_flags: PWRFlags) -> Result<()> { //, vdps_lv: u8, vdns_lv: u8) {
        self.connector.send_command(1)?;
        self.connector.send_data(&pwr_flags.bits.to_be_bytes())
    }

    #[allow(dead_code)]
    pub fn pof_power_off(&mut self) -> Result<()>{
        self.connector.send_command(2)
    }

    #[allow(dead_code)]
    pub fn pfs_power_off_sequence_setting(&mut self, pfs_flags: PFSFlags) -> Result<()> {
        self.connector.send_command(3)?;
        self.connector.send_data(&[pfs_flags.bits])
    }

    #[allow(dead_code)]
    pub fn pon_power_on(&mut self) -> Result<()> {
        self.connector.send_command(4)
    }

    #[allow(dead_code)]
    pub fn btst_booster_soft_start(&mut self, pha: u8, phb: u8, phc: u8) ->Result<()> {
        self.connector.send_command(6)?;
        self.connector.send_data(&[pha, phb, phc])
    }

    #[allow(dead_code)]
    pub fn dslp_deep_sleep(&mut self) -> Result<()> {
        self.connector.send_command(7)?;
        self.connector.send_data(&[0xa5])
    }

    #[allow(dead_code)]
    pub fn drf_display_refresh(&mut self) -> Result<()>{
        self.connector.send_command(0x12)
    }

    #[allow(dead_code)]
    pub fn pll_control(&mut self, frame_rate_code: u8) -> Result<()> {
        self.connector.send_command(0x30)?;
        self.connector.send_data(&[frame_rate_code])
    }

    #[allow(dead_code)]
    pub fn ipc_image_process(&mut self, enabled: bool, line_width: u8) -> Result<()> {
        self.connector.send_command(0x31)?;
        self.connector.send_data(&[(enabled as u8) << 4 | (line_width & 3)])
    }

    #[allow(dead_code)]
    pub fn tse_temperature_sensor_calibration(&mut self, tse: bool, to: u8) -> Result<()> {
        self.connector.send_command(0x41)?;
        self.connector.send_data(&[(tse as u8) << 7 | (to & 0xf)])
    }


    #[allow(dead_code)]
    pub fn cdi_vcom_and_data_interval_settings(&mut self, vbd: u8, ddx: bool, cdi: u8) -> Result<()> {
        self.connector.send_command(0x50)?;
        self.connector.send_data(&[(vbd & 7) << 5 | (ddx as u8) << 4 | cdi & 0xf])
    }

    #[allow(dead_code)]
    pub fn tcon_setting(&mut self, s2g_g2s: u8) -> Result<()> {
        self.connector.send_command(0x60)?;
        self.connector.send_data(&[s2g_g2s])
    }

    #[allow(dead_code)]
    pub fn tres_resolution(&mut self, width: u16, height: u16) -> Result<()> {
        self.connector.send_command(0x61)?;
        // this doesn't seem to match the spec but it's what the demo code does
        self.connector.send_data(&[(width >> 8) as u8, (width & 0xff) as u8, (height >> 8) as u8, (height & 0xff) as u8])
    }

    #[allow(dead_code)]
    pub fn dam_spi_flash_control(&mut self, dam: bool) -> Result<()> {
        self.connector.send_command(0x65)?;
        self.connector.send_data(&[dam as u8])
    }

    #[allow(dead_code)]
    pub fn vcom_dc_setting(&mut self, vv: u8) -> Result<()> {
        self.connector.send_command(0x82)?;
        self.connector.send_data(&[vv])
    }

    // mentioned in the sample code and on page 21 of the spec
    // but no further explanation
    #[allow(dead_code)]
    pub(crate) fn define_flash(&mut self, dunno: u8) -> Result<()> {
        self.connector.send_command(0xe5)?;
        self.connector.send_data(&[dunno])
    }

    #[allow(dead_code)]
    pub fn transmit(&mut self, data: &[u8]) -> Result<()> {
        self.connector.send_command(0x10)?;
        self.connector.send_data(data)
        //    self.connector.send_command(0x11);
    }

    #[allow(dead_code)]
    pub(crate) fn transmit_with<F>(&mut self, repeats: u32, source: F) -> Result<()> where F: Fn(u32) -> u8 {
        self.connector.send_command(0x10)?;
        self.connector.send_data_with(repeats, source)
    }


    #[allow(dead_code)]
    pub fn flash_data(&mut self) -> Result<()> {
        self.connector.send_command(0xb9)
    }

    #[allow(dead_code)]
    pub fn await_ready_state(&mut self) -> Result<()> {
        while self.connector.is_busy()? { self.connector.delay_ms(100)?; }
        Ok(())
    }
}