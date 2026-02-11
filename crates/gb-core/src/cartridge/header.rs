#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
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
    Megabyte8,    // 512 banks
    Megabyte1_1,  // 72 banks
    Megabyte1_2,  // 80 banks
    Megabyte1_5,  // 96 banks
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
            RomSize::Megabyte8 => 512,
            RomSize::Megabyte1_1 => 72,
            RomSize::Megabyte1_2 => 80,
            RomSize::Megabyte1_5 => 96,
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
            0x08 => Ok(RomSize::Megabyte8),
            0x52 => Ok(RomSize::Megabyte1_1),
            0x53 => Ok(RomSize::Megabyte1_2),
            0x54 => Ok(RomSize::Megabyte1_5),
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
            0x05 => Ok(CartridgeType::Mbc2),
            0x06 => Ok(CartridgeType::Mbc2Battery),
            0x0F => Ok(CartridgeType::Mbc3TimerBattery),
            0x10 => Ok(CartridgeType::Mbc3TimerRamBattery),
            0x11 => Ok(CartridgeType::Mbc3),
            0x12 => Ok(CartridgeType::Mbc3Ram),
            0x13 => Ok(CartridgeType::Mbc3RamBattery),
            0x19 => Ok(CartridgeType::Mbc5),
            0x1A => Ok(CartridgeType::Mbc5Ram),
            0x1B => Ok(CartridgeType::Mbc5RamBattery),
            0x1C => Ok(CartridgeType::Mbc5Rumble),
            0x1D => Ok(CartridgeType::Mbc5RumbleRam),
            0x1E => Ok(CartridgeType::Mbc5RumbleRamBattery),
            _ => Err(HeaderError::UnsupportedCartridgeType(byte)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgbSupport {
    DmgOnly,
    CgbCompatible,
    CgbOnly,
}

impl CgbSupport {
    fn from_byte(byte: u8) -> Self {
        // Official values are 0x00 (DMG), 0x80 (CGB-compatible), 0xC0 (CGB-only).
        // Some ROMs set additional bits, so we follow the common "bit 7/6" interpretation.
        if (byte & 0xC0) == 0xC0 {
            Self::CgbOnly
        } else if (byte & 0x80) != 0 {
            Self::CgbCompatible
        } else {
            Self::DmgOnly
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub cartridge_type: CartridgeType,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
    pub cgb_support: CgbSupport,
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
        let cgb_support = CgbSupport::from_byte(rom[0x0143]);

        Ok(Header {
            cartridge_type,
            rom_size,
            ram_size,
            cgb_support,
        })
    }
}
