pub mod header;
pub mod mbc;
pub mod mbc0;
pub mod mbc1;
pub mod mbc3;

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub mbc: Box<dyn mbc::Mbc>,
    pub header: header::Header,
}
