#![no_std]
#[macro_use]
extern crate bitflags;

mod controller;
mod epd7in5_tri_v1;
mod epd7in5_tri_v2;
mod display;

pub use epd7in5_tri_v1::EPaper75TriColour;
pub use epd7in5_tri_v2::EPaper75TriColourV2;
pub use display::EPaperDisplay;
pub use display::DisplayError;
pub use controller::display_connector;
pub use controller::gd7965;
pub use controller::il0371;

#[cfg(test)]
#[macro_use]
extern crate std;
// define empty dbg macro
//