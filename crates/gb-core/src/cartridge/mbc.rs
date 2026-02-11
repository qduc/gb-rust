pub trait Mbc {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, val: u8);
    fn read_ram(&self, ram: &[u8], addr: u16) -> u8;
    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8);

    fn tick(&mut self, _cycles: u32) {}

    fn save_extra(&self) -> Vec<u8> {
        Vec::new()
    }

    fn load_extra(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.is_empty() {
            Ok(())
        } else {
            Err("unexpected mapper save data")
        }
    }
}
