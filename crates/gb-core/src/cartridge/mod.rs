pub mod header;
pub mod mbc;
pub mod mbc0;
pub mod mbc1;
pub mod mbc3;

#[derive(Debug, Clone)]
pub enum CartridgeError {
    HeaderParse(header::HeaderError),
    RomTooSmall { declared: usize, actual: usize },
}

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub mbc: Box<dyn mbc::Mbc>,
    pub header: header::Header,
}

impl Cartridge {
    pub fn from_rom(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        let header = header::Header::parse(&rom).map_err(CartridgeError::HeaderParse)?;

        // Validate ROM size matches header declaration
        let expected_rom_size = header.rom_size.byte_len();
        if rom.len() < expected_rom_size {
            return Err(CartridgeError::RomTooSmall {
                declared: expected_rom_size,
                actual: rom.len(),
            });
        }

        let ram = vec![0u8; header.ram_size.byte_len()];

        let mbc: Box<dyn mbc::Mbc> = match header.cartridge_type {
            header::CartridgeType::RomOnly => Box::new(mbc0::Mbc0),
            header::CartridgeType::Mbc1
            | header::CartridgeType::Mbc1Ram
            | header::CartridgeType::Mbc1RamBattery => Box::new(mbc1::Mbc1::new()),
            header::CartridgeType::Mbc3
            | header::CartridgeType::Mbc3Ram
            | header::CartridgeType::Mbc3RamBattery => Box::new(mbc3::Mbc3::new()),
        };

        Ok(Cartridge {
            rom,
            ram,
            mbc,
            header,
        })
    }
}
