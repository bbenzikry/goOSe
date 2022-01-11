#![no_std]
#![no_main]
#![feature(doc_cfg)]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]
#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![test_runner(crate::kernel_tests::runner)]
#![reexport_test_harness_main = "ktests_launch"]

mod arch;
mod drivers;
mod interrupt_manager;
mod kernel_serial;
mod mm;
mod utils;

#[cfg(test)]
mod kernel_tests;

use core::arch::asm;
use drivers::ns16550::*;
use drivers::plic;

use arch::Architecture;

#[no_mangle]
extern "C" fn k_main(_core_id: usize, device_tree_ptr: usize) -> ! {
    #[cfg(test)]
    ktests_launch();

    let mut arch = arch::new_arch(device_tree_ptr);

    kprintln!("GoOSe is booting");

    // Enable Serial interrupts
    plic::init(plic::QEMU_VIRT_PLIC_BASE_ADDRESS);
    let plic = plic::get();
    if let Err(e) = plic.set_priority(QEMU_VIRT_NS16550_INTERRUPT_NUMBER, 1) {
        kprintln!("{}", e);
    }
    if let Err(e) = plic.enable_interrupt(QEMU_VIRT_NS16550_INTERRUPT_NUMBER, 0) {
        kprintln!("{}", e);
    }
    plic.set_threshold(0);

    let mut memory = mm::MemoryManager::<arch::MemoryImpl>::new(&arch);
    memory.map_address_space();

    kprintln!("[OK] Setup virtual memory");

    let interrupts = interrupt_manager::InterruptManager::<arch::InterruptsImpl>::new();
    interrupts.init_interrupts();

    kprintln!("[OK] Enable interrupts");

    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
