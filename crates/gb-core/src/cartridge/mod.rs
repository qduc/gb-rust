pub mod header;
pub mod mbc;
pub mod mbc0;
pub mod mbc1;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;

use self::header::Header;
use crate::cartridge::mbc::Mbc;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CartridgeError {
    InvalidRomSize(usize),
    InvalidHeader(header::HeaderError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveError {
    Io(String),
    InvalidFormat(&'static str),
    NotBatteryBacked,
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Cartridge {
    #[serde(with = "serde_bytes")]
    pub rom: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub ram: Vec<u8>,
    pub header: Header,
    pub mbc: mbc::MbcEnum,
}

impl Cartridge {
    pub fn from_rom(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        let header = Header::parse(&rom).map_err(CartridgeError::InvalidHeader)?;
        let ram = vec![0; header.ram_size.byte_len()];

        let mbc = match header.cartridge_type {
            header::CartridgeType::RomOnly => mbc::MbcEnum::Mbc0(mbc0::Mbc0::new()),
            header::CartridgeType::Mbc1
            | header::CartridgeType::Mbc1Ram
            | header::CartridgeType::Mbc1RamBattery => mbc::MbcEnum::Mbc1(mbc1::Mbc1::new()),
            header::CartridgeType::Mbc2 | header::CartridgeType::Mbc2Battery => {
                mbc::MbcEnum::Mbc2(mbc2::Mbc2::new())
            }
            header::CartridgeType::Mbc3TimerBattery
            | header::CartridgeType::Mbc3TimerRamBattery
            | header::CartridgeType::Mbc3
            | header::CartridgeType::Mbc3Ram
            | header::CartridgeType::Mbc3RamBattery => mbc::MbcEnum::Mbc3(mbc3::Mbc3::new()),
            header::CartridgeType::Mbc5
            | header::CartridgeType::Mbc5Ram
            | header::CartridgeType::Mbc5RamBattery
            | header::CartridgeType::Mbc5Rumble
            | header::CartridgeType::Mbc5RumbleRam
            | header::CartridgeType::Mbc5RumbleRamBattery => mbc::MbcEnum::Mbc5(mbc5::Mbc5::new()),
        };

        Ok(Self {
            rom,
            ram,
            header,
            mbc,
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

    pub fn save_to_path(&self, path: &Path) -> Result<(), SaveError> {
        if !self.has_battery() {
            return Ok(());
        }

        let mut data = self.ram.clone();
        let extra = self.mbc.save_extra();
        if !extra.is_empty() {
            data.extend_from_slice(b"GBSV1");
            data.extend_from_slice(&(extra.len() as u32).to_le_bytes());
            data.extend_from_slice(&extra);
        }

        std::fs::write(path, data).map_err(|e| SaveError::Io(e.to_string()))
    }

    pub fn load_from_path(&mut self, path: &Path) -> Result<(), SaveError> {
        if !self.has_battery() {
            return Ok(());
        }
        if !path.exists() {
            return Ok(());
        }

        let data = std::fs::read(path).map_err(|e| SaveError::Io(e.to_string()))?;

        // Basic verification: data must be at least as large as RAM
        let ram_len = self.ram.len();
        if data.len() < ram_len {
            // If save file is smaller than RAM, copy what we can, but likely invalid/partial
            if ram_len > 0 {
                let copy_len = data.len();
                self.ram[..copy_len].copy_from_slice(&data[..copy_len]);
            }
            return Ok(());
        }

        // Load RAM
        if ram_len > 0 {
            self.ram.copy_from_slice(&data[..ram_len]);
        }

        // Check for footer
        let trailer = &data[ram_len..];
        if trailer.is_empty() {
            return self.mbc.load_extra(&[]).map_err(SaveError::InvalidFormat);
        }

        if trailer.len() < 9 {
            // Too short for header, ignore
            return Ok(());
        }

        if &trailer[..5] != b"GBSV1" {
            // Not our format, maybe raw RAM dump.
            return Ok(());
        }

        let len_bytes = [trailer[5], trailer[6], trailer[7], trailer[8]];
        let extra_len = u32::from_le_bytes(len_bytes) as usize;

        if trailer.len() < 9 + extra_len {
            return Err(SaveError::InvalidFormat("save trailer truncated"));
        }

        self.mbc
            .load_extra(&trailer[9..9 + extra_len])
            .map_err(SaveError::InvalidFormat)
    }
}
