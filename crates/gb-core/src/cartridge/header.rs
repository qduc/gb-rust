#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomSize {
    Kilobytes32,  // 1 bank
    Kilobytes64,  // 2 banks
    Kilobytes128, // 4 banks
    Kilobytes256, // 8 banks
    Kilobytes512, // 16 banks
    Megabyte1,    // 32 banks
    Megabyte2,    // 64 banks
    Megabyte4,    // 128 banks
}

impl RomSize {
    pub fn bank_count(self) -> usize {
        match self {
            RomSize::Kilobytes32 => 1,
            RomSize::Kilobytes64 => 2,
            RomSize::Kilobytes128 => 4,
            RomSize::Kilobytes256 => 8,
            RomSize::Kilobytes512 => 16,
            RomSize::Megabyte1 => 32,
            RomSize::Megabyte2 => 64,
            RomSize::Megabyte4 => 128,
        }
    }

    pub fn byte_len(self) -> usize {
        self.bank_count() * 0x4000
    }

    fn from_byte(byte: u8) -> Result<Self, HeaderError> {
        match byte {
            0x00 => Ok(RomSize::Kilobytes32),
            0x01 => Ok(RomSize::Kilobytes64),
            0x02 => Ok(RomSize::Kilobytes128),
            0x03 => Ok(RomSize::Kilobytes256),
            0x04 => Ok(RomSize::Kilobytes512),
            0x05 => Ok(RomSize::Megabyte1),
            0x06 => Ok(RomSize::Megabyte2),
            0x07 => Ok(RomSize::Megabyte4),
            _ => Err(HeaderError::UnsupportedRomSize(byte)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RamSize {
    None,
    Kilobytes8,
    Kilobytes32,
    Kilobytes128,
    Kilobytes64,
}

impl RamSize {
    pub fn byte_len(self) -> usize {
        match self {
            RamSize::None => 0,
            RamSize::Kilobytes8 => 0x2000,
            RamSize::Kilobytes32 => 0x8000,
            RamSize::Kilobytes128 => 0x20000,
            RamSize::Kilobytes64 => 0x10000,
        }
    }

    fn from_byte(byte: u8) -> Result<Self, HeaderError> {
        match byte {
            0x00 => Ok(RamSize::None),
            0x02 => Ok(RamSize::Kilobytes8),
            0x03 => Ok(RamSize::Kilobytes32),
            0x04 => Ok(RamSize::Kilobytes128),
            0x05 => Ok(RamSize::Kilobytes64),
            _ => Err(HeaderError::UnsupportedRamSize(byte)),
        }
    }
}

impl CartridgeType {
    fn from_byte(byte: u8) -> Result<Self, HeaderError> {
        match byte {
            0x00 => Ok(CartridgeType::RomOnly),
            0x01 => Ok(CartridgeType::Mbc1),
            0x02 => Ok(CartridgeType::Mbc1Ram),
            0x03 => Ok(CartridgeType::Mbc1RamBattery),
            0x11 => Ok(CartridgeType::Mbc3),
            0x12 => Ok(CartridgeType::Mbc3Ram),
            0x13 => Ok(CartridgeType::Mbc3RamBattery),
            _ => Err(HeaderError::UnsupportedCartridgeType(byte)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub cartridge_type: CartridgeType,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
}

#[derive(Debug, Clone)]
pub enum HeaderError {
    RomTooSmall,
    UnsupportedCartridgeType(u8),
    UnsupportedRomSize(u8),
    UnsupportedRamSize(u8),
}

impl Header {
    pub fn parse(rom: &[u8]) -> Result<Self, HeaderError> {
        if rom.len() < 0x014A {
            return Err(HeaderError::RomTooSmall);
        }

        let cartridge_type = CartridgeType::from_byte(rom[0x0147])?;
        let rom_size = RomSize::from_byte(rom[0x0148])?;
        let ram_size = RamSize::from_byte(rom[0x0149])?;

        Ok(Header {
            cartridge_type,
            rom_size,
            ram_size,
        })
    }
}
