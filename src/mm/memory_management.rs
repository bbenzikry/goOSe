use crate::arch;

use crate::mm::{
    MemoryManager, Permissions, PAddr, VAddr,
};
use crate::mm;

pub struct MemoryManagement<'alloc, T: arch::ArchitectureMemory> {
    arch: &'alloc mut T,
}

impl<'alloc, T: arch::ArchitectureMemory> MemoryManagement<'alloc, T> {
    pub fn new() -> Self {
        let page_allocator = mm::get_global_allocator();
        let arch_mem = T::new(page_allocator);

        Self {
            arch: arch_mem,
        }
    }
}

impl<T: arch::ArchitectureMemory> MemoryManager for MemoryManagement<'_, T> {
    fn map(&mut self, phys: PAddr, virt: VAddr, perms: Permissions) {
        self.arch
            .map(mm::get_global_allocator(), phys.into(), virt.into(), perms)
    }

    fn reload_page_table(&mut self) {
        self.arch.reload();
    }
}
