// OAM DMA (and later HDMA for GBC)

pub const OAM_DMA_BYTES: u16 = 0x00A0;
pub const OAM_DMA_CYCLES_PER_BYTE: u32 = 4;

#[derive(Copy, Clone, Debug, Default)]
pub struct OamDma {
    active: bool,
    source_base: u16,
    next_byte: u16,
    cycle_budget: u32,
}

impl OamDma {
    pub fn start(&mut self, page: u8) {
        self.active = true;
        self.source_base = (page as u16) << 8;
        self.next_byte = 0;
        self.cycle_budget = 0;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn blocks_cpu_addr(&self, addr: u16) -> bool {
        self.active && !(0xFF80..=0xFFFE).contains(&addr)
    }

    pub fn add_cycles(&mut self, cycles: u32) {
        if !self.active {
            return;
        }
        self.cycle_budget = self.cycle_budget.saturating_add(cycles);
    }

    pub fn pop_transfer(&mut self) -> Option<(u16, usize)> {
        if !self.active || self.cycle_budget < OAM_DMA_CYCLES_PER_BYTE {
            return None;
        }

        self.cycle_budget -= OAM_DMA_CYCLES_PER_BYTE;

        let src = self.source_base.wrapping_add(self.next_byte);
        let dst = self.next_byte as usize;
        self.next_byte = self.next_byte.wrapping_add(1);

        if self.next_byte >= OAM_DMA_BYTES {
            self.active = false;
            self.cycle_budget = 0;
        }

        Some((src, dst))
    }
}
