pub mod header;
pub mod mbc;
pub mod mbc0;
pub mod mbc1;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;

use std::path::Path;

#[derive(Debug, Clone)]
pub enum CartridgeError {
    HeaderParse(header::HeaderError),
    RomTooSmall { declared: usize, actual: usize },
}

#[derive(Debug)]
pub enum SaveError {
    NotBatteryBacked,
    Io(std::io::Error),
    InvalidFormat(&'static str),
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
            header::CartridgeType::Mbc2 | header::CartridgeType::Mbc2Battery => {
                Box::new(mbc2::Mbc2::new())
            }
            header::CartridgeType::Mbc3TimerBattery
            | header::CartridgeType::Mbc3TimerRamBattery
            | header::CartridgeType::Mbc3
            | header::CartridgeType::Mbc3Ram
            | header::CartridgeType::Mbc3RamBattery => Box::new(mbc3::Mbc3::new()),
            header::CartridgeType::Mbc5
            | header::CartridgeType::Mbc5Ram
            | header::CartridgeType::Mbc5RamBattery
            | header::CartridgeType::Mbc5Rumble
            | header::CartridgeType::Mbc5RumbleRam
            | header::CartridgeType::Mbc5RumbleRamBattery => Box::new(mbc5::Mbc5::new()),
        };

        Ok(Cartridge {
            rom,
            ram,
            mbc,
            header,
        })
    }

    pub fn has_battery(&self) -> bool {
        matches!(
            self.header.cartridge_type,
            header::CartridgeType::Mbc1RamBattery
                | header::CartridgeType::Mbc2Battery
                | header::CartridgeType::Mbc3TimerBattery
                | header::CartridgeType::Mbc3TimerRamBattery
                | header::CartridgeType::Mbc3RamBattery
                | header::CartridgeType::Mbc5RamBattery
                | header::CartridgeType::Mbc5RumbleRamBattery
        )
    }

    pub fn export_save_data(&self) -> Option<Vec<u8>> {
        if !self.has_battery() {
            return None;
        }

        let extra = self.mbc.save_extra();
        let mut out = self.ram.clone();
        if !extra.is_empty() {
            out.extend_from_slice(b"GBSV1");
            out.extend_from_slice(&(extra.len() as u32).to_le_bytes());
            out.extend_from_slice(&extra);
        }
        Some(out)
    }

    pub fn import_save_data(&mut self, data: &[u8]) -> Result<(), SaveError> {
        if !self.has_battery() {
            return Err(SaveError::NotBatteryBacked);
        }

        let ram_len = self.ram.len();
        if data.len() < ram_len {
            return Err(SaveError::InvalidFormat("save file smaller than RAM size"));
        }
        if ram_len > 0 {
            self.ram.copy_from_slice(&data[..ram_len]);
        }

        let trailer = &data[ram_len..];
        if trailer.is_empty() {
            return self.mbc.load_extra(&[]).map_err(SaveError::InvalidFormat);
        }

        if trailer.len() < 9 {
            return Err(SaveError::InvalidFormat("save trailer is truncated"));
        }
        if &trailer[..5] != b"GBSV1" {
            return Err(SaveError::InvalidFormat("unknown save trailer magic"));
        }

        let extra_len =
            u32::from_le_bytes([trailer[5], trailer[6], trailer[7], trailer[8]]) as usize;
        if trailer.len() != 9 + extra_len {
            return Err(SaveError::InvalidFormat("save trailer length mismatch"));
        }
        self.mbc
            .load_extra(&trailer[9..])
            .map_err(SaveError::InvalidFormat)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), SaveError> {
        let Some(data) = self.export_save_data() else {
            return Err(SaveError::NotBatteryBacked);
        };
        std::fs::write(path, data).map_err(SaveError::Io)
    }

    pub fn load_from_path(&mut self, path: &Path) -> Result<(), SaveError> {
        if !path.exists() {
            return Ok(());
        }
        let data = std::fs::read(path).map_err(SaveError::Io)?;
        self.import_save_data(&data)
    }
}
