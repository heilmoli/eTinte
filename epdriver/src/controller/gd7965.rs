use crate::controller::display_connector::{DisplayConnector, Result};

// driver

// http://www.e-paper-display.com/download_detail/downloadsId=536.html
// seems to be same as UC8159C (https://www.buydisplay.com/download/ic/UC8159C.pdf)
bitflags! {
    pub struct PSRFlags: u8 {
        const REG   = 0b0010_0000;
        const KW_R  = 0b0001_0000;
        const UD    = 0b0000_1000;
        const SHL   = 0b0000_0100;
        const SHD_N = 0b0000_0010;
        const RST_N = 0b0000_0001;
    }
}

bitflags! {
    pub struct PWRFlags: u16 {
        const BD_EN         = 0b0001_0000_0000_0000;
        const VSR_EN        = 0b0000_0100_0000_0000;
        const VS_EN         = 0b0000_0010_0000_0000;
        const VG_EN         = 0b0000_0001_0000_0000;

        const VPP_EN        = 0b0000_0000_1000_0000;
        const VCOM_SLEW     = 0b0000_0000_0001_0000;
        const VG_LVL_9V     = 0b0000_0000_0000_0000;
        const VG_LVL_10V    = 0b0000_0000_0000_0001;
        const VG_LVL_11V    = 0b0000_0000_0000_0010;
        const VG_LVL_12V    = 0b0000_0000_0000_0011;
        const VG_LVL_17V    = 0b0000_0000_0000_0100;
        const VG_LVL_18V    = 0b0000_0000_0000_0101;
        const VG_LVL_19V    = 0b0000_0000_0000_0110;
        const VG_LVL_20V    = 0b0000_0000_0000_0111;
    }
}

pub(crate) struct GD7965<T> where T: DisplayConnector {
    connector: T
}

impl<T> GD7965<T> where T: DisplayConnector {
    pub fn new(connector: T) -> GD7965<T> {
        GD7965 {
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
    pub(crate) fn pwr_power_setting(&mut self, pwr_flags: PWRFlags, vdh_lvl_volts:f32, vdl_lvl_volts:f32, vdhr_lvl_volts:f32 ) -> Result<()> { //, vdps_lv: u8, vdns_lv: u8) {

        assert!(vdh_lvl_volts>=2.4&& vdh_lvl_volts<=15.0);
        assert!(vdl_lvl_volts>=-15.0&& vdl_lvl_volts<=-2.4);
        assert!(vdhr_lvl_volts>=2.4&& vdhr_lvl_volts<=15.0);

        let vdh_lvl = 0x3f&(0.5 + (vdh_lvl_volts-2.4) / 0.2) as u8;
        let vdl_lvl = 0x3f&(0.5 + (-vdl_lvl_volts-2.4) / 0.2) as u8;
        let vdhr_lvl = 0x3f&(0.5 + (vdhr_lvl_volts-2.4) / 0.2) as u8;

        self.connector.send_command(1)?;
        self.connector.send_data(&pwr_flags.bits.to_be_bytes())?;
        self.connector.send_data(&[ vdh_lvl, vdl_lvl, vdhr_lvl])
    }

    #[allow(dead_code)]
    pub fn pof_power_off(&mut self) -> Result<()> {
        self.connector.send_command(2)
    }

    #[allow(dead_code)]
    pub fn pfs_power_off_sequence_setting(&mut self, t_vds_off: u8) -> Result<()> {
        self.connector.send_command(3)?;
        self.connector.send_data(&[(t_vds_off & 3) <<4 ])
    }

    #[allow(dead_code)]
    pub fn pon_power_on(&mut self) -> Result<()> {
        self.connector.send_command(4)
    }

    #[allow(dead_code)]
    pub fn btst_booster_soft_start(&mut self, pha: u8, phb: u8, phc: u8, phc2en: bool, phc2 : u8) -> Result<()> {
        self.connector.send_command(6)?;
        self.connector.send_data(&[pha, phb, phc, (phc2en as u8) << 7 | phc2])
    }

    #[allow(dead_code)]
    pub fn dslp_deep_sleep(&mut self) -> Result<()> {
        self.connector.send_command(7)?;
        self.connector.send_data(&[0xa5])
    }

    #[allow(dead_code)]
    pub fn duspi_dual_spi_mode(&mut self, mm_en:bool, duspi_en:bool) -> Result<()> {
        self.connector.send_command(0x15)?;
        self.connector.send_data(&[(mm_en as u8) << 5 | (duspi_en as u8) << 4])
    }

    #[allow(dead_code)]
    pub fn drf_display_refresh(&mut self) -> Result<()> {
        self.connector.send_command(0x12)
    }

    #[allow(dead_code)]
    pub fn pll_control(&mut self, frs: u8) -> Result<()> {
        self.connector.send_command(0x30)?;
        self.connector.send_data(&[frs])
    }

    #[allow(dead_code)]
    pub fn cdi_vcom_and_data_interval_settings(&mut self, bdz: bool, bdv: u8, n2ocp: bool, ddx: u8, cdi: u8) -> Result<()> {
        self.connector.send_command(0x50)?;
        self.connector.send_data(&[(bdz as u8) << 7 | (bdv & 3) << 4 | (n2ocp as u8) << 3 | ddx & 3, cdi & 7])
    }

    #[allow(dead_code)]
    pub fn tcon_setting(&mut self, s2g: u8, g2s: u8) -> Result<()> {
        self.connector.send_command(0x60)?;
        self.connector.send_data(&[(s2g & 7) << 4 | g2s & 7])
    }

    #[allow(dead_code)]
    pub fn tres_resolution(&mut self, width: u16, height: u16) -> Result<()> {
        self.connector.send_command(0x61)?;
        // this doesn't seem to match the spec but it's what the demo code does
        self.connector.send_data(&[(width >> 8) as u8, (width & 0xff) as u8, (height >> 8) as u8, (height & 0xff) as u8])
    }

    #[allow(dead_code)]
    pub fn gss_gate_source_start_setting(&mut self, hst: u16, vst: u16) -> Result<()> {
        self.connector.send_command(0x65)?;
        self.connector.send_data(&[
            0x3 & (hst >> 8) as u8,
            0xf1 & hst as u8,
            0x3 & (vst >> 8) as u8,
            vst as u8])
    }

    #[allow(dead_code)]
    pub fn vcom_dc_setting(&mut self, vdcs: u8) -> Result<()> {
        self.connector.send_command(0x82)?;
        self.connector.send_data(&[vdcs])
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
        self.connector.send_data_with(repeats, &source)

    }

    #[allow(dead_code)]
    pub(crate) fn transmit_with2<F>(&mut self, repeats: u32, source: F) -> Result<()> where F: Fn(u32) -> u8 {
        self.connector.send_command(0x13)?;
        self.connector.send_data_with(repeats, source)

    }

    #[allow(dead_code)]
    pub fn flash_data(&mut self) -> Result<()> {
        self.connector.send_command(0xb9)
    }

    #[allow(dead_code)]
    pub fn await_ready_state(&mut self) -> Result<()> {
        self.connector.send_command(0x71)?;
        while self.connector.is_busy()? {
            self.connector.delay_ms(100)?;
            self.connector.send_command(0x71)?;
        }
        /*
                self.connector.send_command(0x71)?;
        while self.connector.is_busy()? { self.connector.delay_ms(1)?; }
        Ok(())
         */
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;
    use crate::controller::display_connector::{DisplayConnector,Result};
    use crate::controller::gd7965::{GD7965, PWRFlags};
    use core::cell::RefCell;


    struct DataRecorder<'a> {
        cmds: & 'a mut Vec<u8>,
        data : & 'a mut Vec<u8>
    }

    impl DisplayConnector for DataRecorder<'_> {
        fn reset(&mut self) -> Result<()> {
            unimplemented!()
        }

        fn is_busy(&self) -> Result<bool> {
            unimplemented!()
        }

        fn send_command(&mut self, command: u8) -> Result<()> {
            self.cmds.push(command);
            Ok(())
        }

        fn send_data_with<F>(&mut self, repeats: u32, source: F) -> Result<()> where F: Fn(u32) -> u8 {
            unimplemented!()
        }

        fn send_data(&mut self, data: &[u8]) -> Result<()> {
            self.data.extend(data.iter());
            Ok(())
        }

        fn delay_ms(&mut self, ms: u16) -> Result<()> {
            unimplemented!()
        }
    }

    #[test]
    fn test_pwr_settings() {
        let mut cmd_bytes = Vec::new();
        let mut data_bytes = Vec::new();
        let mut driver = GD7965::new(DataRecorder { cmds : & mut cmd_bytes, data: & mut data_bytes });

        driver.pwr_power_setting(PWRFlags::VSR_EN|PWRFlags::VS_EN|PWRFlags::VG_EN|PWRFlags::VG_LVL_20V,
        15.0,-15.0, 3.0);

        assert_eq!( cmd_bytes.len(), 1 as usize);
        assert_eq!( data_bytes.len(), 5 as usize);
        assert_eq!( cmd_bytes[0], 0x01 );
        assert_eq!( data_bytes[0], 0x07 );
        assert_eq!( data_bytes[1], 0x07 );
        assert_eq!( data_bytes[2], 0x3f );
        assert_eq!( data_bytes[3], 0x3f );
        assert_eq!( data_bytes[4], 0x03 );
    }

    #[test]
    fn test_pwr_on() {
        let mut cmd_bytes = Vec::new();
        let mut data_bytes = Vec::new();
        let mut driver = GD7965::new(DataRecorder { cmds : & mut cmd_bytes, data: & mut data_bytes });

        driver.pon_power_on();

        assert_eq!( cmd_bytes.len(), 1 as usize);
        assert_eq!( data_bytes.len(), 0 as usize );
        assert_eq!( cmd_bytes[0], 4);
    }

    #[test]
    fn test_tres() {
        let mut cmd_bytes = Vec::new();
        let mut data_bytes = Vec::new();
        let mut driver = GD7965::new(DataRecorder { cmds : & mut cmd_bytes, data: & mut data_bytes });

        driver.tres_tcon_resolution(800,480);

        assert_eq!(cmd_bytes.len(), 1);
        assert_eq!(data_bytes.len(), 4);
        assert_eq!(cmd_bytes[0], 0x61);
        assert_eq!(data_bytes[0], 0x03);
        assert_eq!(data_bytes[1], 0x20);
        assert_eq!(data_bytes[2], 0x01);
        assert_eq!(data_bytes[3], 0xe0);
    }

    #[test]
    fn test_duspi_dual_spi_mode() {
        let mut cmd_bytes = Vec::new();
        let mut data_bytes = Vec::new();
        let mut driver = GD7965::new(DataRecorder { cmds : & mut cmd_bytes, data: & mut data_bytes });

        driver.duspi_dual_spi_mode(true,false);

        assert_eq!( cmd_bytes.len(), 1 as usize);
        assert_eq!( data_bytes.len(), 1 as usize );
        assert_eq!( cmd_bytes[0], 0x15);
        assert_eq!( data_bytes[0], 0x20);
    }

    #[test]
    fn test_cdi_vcom_and_data_interval_settings() {
        let mut cmd_bytes = Vec::new();
        let mut data_bytes = Vec::new();
        let mut driver = GD7965::new(DataRecorder { cmds : & mut cmd_bytes, data: & mut data_bytes });

        driver.cdi_vcom_and_data_interval_settings(false, 1, false, 1, 7);

        assert_eq!( cmd_bytes.len(), 1 as usize);
        assert_eq!( data_bytes.len(), 2 as usize );
        assert_eq!( cmd_bytes[0], 0x50);
        assert_eq!( data_bytes[0], 0x11);
        assert_eq!( data_bytes[1], 0x07);
    }

    #[test]
    fn test_tcon_setting() {
        let mut cmd_bytes = Vec::new();
        let mut data_bytes = Vec::new();
        let mut driver = GD7965::new(DataRecorder { cmds : & mut cmd_bytes, data: & mut data_bytes });

        driver.tcon_setting(2,2);

        assert_eq!( cmd_bytes.len(), 1 as usize);
        assert_eq!( data_bytes.len(), 1 as usize );
        assert_eq!( cmd_bytes[0], 0x60);
        assert_eq!( data_bytes[0], 0x22);
    }


}

