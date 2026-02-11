pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
pub const FRAMEBUFFER_LEN: usize = LCD_WIDTH * LCD_HEIGHT;

pub type Framebuffer = [u32; FRAMEBUFFER_LEN];

pub mod oam;
#[allow(clippy::module_inception)]
pub mod ppu;
pub mod render;

pub use ppu::Ppu;
