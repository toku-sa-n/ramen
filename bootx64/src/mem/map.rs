struct MapInfo {
    virt: usize,
    phys: usize,
    bytes: usize,
}

impl MapInfo {
    fn new(virt: usize, phys: usize, bytes: usize) -> Self {
        Self { virt, phys, bytes }
    }

    fn map(&self, mem_map: &mut [boot::MemoryDescriptor]) -> () {
        map_virt_to_phys(self.virt, self.phys, self.bytes, mem_map);
    }
}

/// (*mut u8, usize): (address to memory map, the size of memory map)
pub fn generate_map(system_table: &SystemTable<Boot>) -> (*mut u8, usize) {
    // Using returned value itself causes bufer too small erorr.
    // Doubling should solve this.
    let memory_map_size = system_table.boot_services().memory_map_size() * 2;

    info!("memory_map_size: {}", memory_map_size);

    // The last unwrap is for `Completion` type. Because the first `expect` is for `Result` type,
    // it is not needed to change `unwrap` to `expect_succes`.
    let memory_map_buf = system_table
        .boot_services()
        .allocate_pool(MemoryType::LOADER_DATA, memory_map_size)
        .expect("Failed to allocate memory for memory map")
        .unwrap();

    system_table
        .boot_services()
        .memory_map(unsafe { core::slice::from_raw_parts_mut(memory_map_buf, memory_map_size) })
        .expect("Failed to get memory map")
        .unwrap();

    (memory_map_buf, memory_map_size)
}
