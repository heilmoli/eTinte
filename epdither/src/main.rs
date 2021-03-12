
use image;
use dither::ditherer::{Dither};
use std::path::Path;
use image::{GenericImageView, DynamicImage};
use dither::color::palette;
use dither::{clamp_f64_to_u8, ditherer};
use epdriver::{EPaper75TriColour, EPaper75TriColourV2, EPaperDisplay, DisplayError};

use linux_embedded_hal::{spidev::{SpiModeFlags, SpidevOptions}, Spidev, CdevPin, Delay, SysfsPin};
use image::imageops::{FilterType};
use  rand::prelude::*;
use std::path::PathBuf;
use gumdrop::Options;
use std::str::FromStr;
use crate::CropAlign::{TopLeft, Centre};
use dither::prelude::{Img, RGB};
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use linux_embedded_hal::sysfs_gpio::{Pin, Direction};


#[derive(Debug)]
enum CropAlign {
    TopLeft,
    BottomRight,
    Centre
}

#[derive(Debug)]
enum DisplayType {
    YellowBlack75V1,
    YellowBlack75V2
}

impl FromStr for CropAlign {
    type Err = String;
    fn from_str(crop: &str) -> Result<Self, Self::Err> {
        match crop.to_lowercase().as_str() {
            "topleft"|"tl" => Ok(Self::TopLeft),
            "bottomright"|"br" => Ok(Self::BottomRight),
            "centre"|"c" => Ok(Centre),
            _ => Err(format!("failed to parse crop option from {}", crop )),
        }
    }
}

#[derive(Debug, Options)]
struct CommandLineOptions {

    #[options(help = "print help message")]
    help: bool,

    #[options(help = "more information on stdout")]
    verbose: bool,

    #[options(help = "specify image alignment if image needs to be cropped", meta="[topleft|tl|bottomright|br|centre|c]" )]
    crop_align: Option<CropAlign>,

    #[options(help = "flip image vertically - around the horizontal axis")]
    flipv: bool,

    #[options(help = "flip image horizontal - around the vertical axis")]
    fliph: bool,

    /// Files to process
    #[options(free)]
    image_file: PathBuf,
}

fn main() {
    let opt = CommandLineOptions::parse_args_default_or_exit();
    println!("{:?}", opt);

    // TODO: oether display
    // provide 'middle colour reference'

    if opt.verbose {
        println!("initializing display")
    }
    let mut display = init_display(opt.verbose);
    if opt.verbose {
        println!("init done display")
    }

    let mut im = image::open(&Path::new(&opt.image_file)).unwrap();

    let display_ar = display.width() as f32 /display.height() as f32;

    let img_ar = im.width() as f32 / im.height() as f32;
    print!("{}x{} vs {}x{} display_ar={} img_ar={}", display.width(), display.height(), im.width(), im.height(), display_ar, img_ar);
    let crop = if img_ar > display_ar {
        let target_width =  (im.height() as f32 * display_ar) as u32;
        let x_off = match opt.crop_align {
            Option::Some(CropAlign::TopLeft) => 0,
            Option::Some(CropAlign::BottomRight) => im.width() - target_width,
            _ => (im.width() - target_width)/2
        };
        println!("cropA {},{},{},{}",x_off, 0, target_width, im.height());
        im.crop( x_off, 0, target_width, im.height())
    } else {
        let target_height=  (im.width() as f32 / display_ar) as u32;
        let y_off = match &opt.crop_align {
            Option::Some(CropAlign::TopLeft) => 0,
            Option::Some(CropAlign::BottomRight) => im.height() - target_height,
            _ => (im.height() - target_height)/2
        };
        println!("cropB {},{},{},{}",0,y_off, im.width(), target_height);
        im.crop(0,y_off, im.width(), target_height)
        //  im.crop(0,0, im.width(), target_height)
    };

    let resized_im = crop.resize(display.width() as u32, display.height() as u32, FilterType::Lanczos3);

    println!("sized: {}x{}", resized_im.width(), resized_im.height());

    let final_im = if opt.flipv {
        if opt.fliph { resized_im.flipv().fliph() } else { resized_im.flipv() }
    } else {
        if opt.fliph { resized_im.fliph() } else { resized_im }
    };

    let img = dither_image(final_im).unwrap();

    display.push_image_with(|x,y| {
        img.get((x,y)).map(|rgb| match rgb.0 { x if x < 85  => 0, x if x < 170 => 1, _ => 2 }).unwrap_or( 0)
    }).expect("could not push image to display");
    println!("all done");
}

fn init_display(verbose:bool) -> impl EPaperDisplay {
    let mut spi = Spidev::open("/dev/spidev0.0").expect("failed to open spi device");
    if verbose {
        println!("spi open")
    }

    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");
    if verbose {
        println!("spi done")
    }

    let mut gpio_chip = Chip::new("/dev/gpiochip0").expect("failed to open gpio device");

    let reset_line_handle = gpio_chip.get_line(17).expect("failed to get reset line").request(LineRequestFlags::OUTPUT, 0,"epd reset write" ).unwrap();
    if verbose {
        println!("rst line handle done")
    }
    let dc_line_handle = gpio_chip.get_line(25).expect("failed to get dc line").request(LineRequestFlags::OUTPUT, 0,"epd dc write" ).unwrap();
    if verbose {
        println!("rst line handle done")
    }
    let busy_line_handle = gpio_chip.get_line(24).expect("failed to get busy line").request(LineRequestFlags::INPUT, 0,"epd busy read" ).unwrap();
    if verbose {
        println!("busy line handle done")
    }
    let rst = CdevPin::new(reset_line_handle).unwrap();
    rst.set_value(1).expect("rst set value failed");
    if verbose {
        println!("rst ready")
    }
    let dc = CdevPin::new(dc_line_handle).unwrap();
    dc.set_value(1).expect("dc set value failed");
    if verbose {
        println!("dc ready")
    }
    let busy = CdevPin::new(busy_line_handle).unwrap();
    if verbose {
        println!("pins ready done")
    }

    let mut display = EPaper75TriColourV2::new(
        spi,
        rst,
        dc,
        busy,
        Delay {},
        1024);
    display.init().expect("failed to init display");
    if verbose {
        println!("display ready")
    }
    display
}

fn dither_image(d_img : DynamicImage)  -> dither::Result<Img<RGB<u8>>> {

    let rgb_img = d_img.to_rgb();
    let rgb_buffer: Vec<RGB<u8>> = rgb_img.pixels().map(|p| RGB::from(p.0)).collect();

    let img= Img::<RGB<u8>>::new(rgb_buffer, d_img.width()).expect("dither image instantiation").convert_with(|rgb| rgb.convert_with(f64::from));

    //  TODO:  get this from the driver
    let pallet = [
        RGB(0x00,0x00,0x00),
        //  RGB(0x80,0x5F,0x10),
        RGB(0x93,0x78,0x00),
        RGB(0xFF,0xFF, 0xFF)];


    let dithered_img = ditherer::ATKINSON.dither(img, palette::quantize(&pallet)).convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8));
    // .convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8));

    // The dimensions method returns the images width and height
    println!("dimensions {}x{} {:?} {}", dithered_img.width(), dithered_img.height(), dithered_img.size(), dithered_img.len());

    return Result::Ok(dithered_img);
}
