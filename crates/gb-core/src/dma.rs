// OAM DMA (and later HDMA for GBC)

use crate::bus::Bus;

/// Perform a DMG OAM DMA transfer (FF46).
///
/// Copies 0xA0 bytes from (page << 8) to OAM (FE00..FE9F).
pub fn oam_dma(bus: &mut Bus, page: u8) {
    let src = (page as u16) << 8;
    for i in 0..0xA0u16 {
        let v = bus.read8(src.wrapping_add(i));
        bus.oam[i as usize] = v;
    }
}
