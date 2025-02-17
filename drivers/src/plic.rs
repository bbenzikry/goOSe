//! Driver fot the RISC-V Platform-Level Interrupt Controller
//! <https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc>

use super::Driver;

pub const QEMU_VIRT_PLIC_BASE_ADDRESS: usize = 0xc000000;

const PLIC_ENABLE_OFFSET: usize = 0x002080;
const PLIC_THRESHOLD_OFFSET: usize = 0x201000;
const PLIC_NUMBER_SOURCES: u16 = 1024;
const PLIC_NUMBER_INTERRUPT_SOURCE_BY_REGISTER: u8 = 32;
const PLIC_NUMBER_SOURCE_REGISTER: u16 =
    PLIC_NUMBER_SOURCES / PLIC_NUMBER_INTERRUPT_SOURCE_BY_REGISTER as u16;
const PLIC_MAX_CONTEXT: u16 = 0x3e00;
const PLIC_CLAIM_OFFSET: usize = 0x201004;

static mut PLIC: Option<Plic> = None;

pub struct Plic {
    base_register_address: usize,
    source_handler: [fn(); PLIC_NUMBER_SOURCES as usize],
}

impl Plic {
    pub fn new(base_register_address: usize) -> Plic {
        Self {
            base_register_address,
            source_handler: [not_registered; PLIC_NUMBER_SOURCES as usize],
        }
    }

    pub fn set_threshold(&self, threshold: u8) {
        unsafe {
            let addr = (self.base_register_address + PLIC_THRESHOLD_OFFSET) as *mut u32;
            addr.write_volatile(threshold as u32);
        }
    }

    pub fn enable_interrupt(&self, id: u16, hart: u16) -> Result<(), &'static str> {
        if id >= PLIC_NUMBER_SOURCES {
            return Err("enable_interrupt: Id is higher than PLIC_MAX_INTERRUPT_SOURCE");
        }

        if hart >= PLIC_MAX_CONTEXT {
            return Err("enable_interrupt: hart is higher than PLIC_MAX_CONTEXT");
        }

        let source_offset = (id / PLIC_NUMBER_INTERRUPT_SOURCE_BY_REGISTER as u16
            + hart * PLIC_NUMBER_SOURCE_REGISTER) as usize;
        let id_shift = 1 << (id % PLIC_NUMBER_INTERRUPT_SOURCE_BY_REGISTER as u16);

        unsafe {
            let addr =
                (self.base_register_address + PLIC_ENABLE_OFFSET + source_offset) as *mut u32;
            let current_interrupt = core::ptr::read_volatile(addr);
            addr.write_volatile(current_interrupt | id_shift);
        }

        Ok(())
    }

    pub fn set_priority(&self, id: u16, priority: u32) -> Result<(), &'static str> {
        if id >= PLIC_NUMBER_SOURCES {
            return Err("set_priority: Id is higher than PLIC_MAX_INTERRUPT_SOURCE");
        }

        unsafe {
            let addr = (self.base_register_address + (id * 4) as usize) as *mut u32;
            addr.write_volatile(priority);
        }

        Ok(())
    }

    pub fn claim(&self) -> u32 {
        unsafe {
            let addr = (self.base_register_address + PLIC_CLAIM_OFFSET) as *mut u32;
            addr.read_volatile()
        }
    }

    pub fn complete(&self, source: u32) {
        unsafe {
            let addr = (self.base_register_address + PLIC_CLAIM_OFFSET) as *mut u32;
            addr.write_volatile(source);
        }
    }

    pub fn _register_handler(&mut self, id: u16, handler: fn()) {
        self.source_handler[id as usize] = handler;
    }
}

pub fn init(base_register_address: usize) {
    unsafe {
        PLIC = Some(Plic::new(base_register_address));
    }
}

pub fn get() -> &'static mut Plic {
    let plic = unsafe { &mut PLIC };

    match plic.as_mut() {
        Some(plic_ref) => plic_ref,
        None => unreachable!("PLIC should have been initialized at this point"),
    }
}

fn not_registered() {}

impl Driver for Plic {
    fn get_address_range(&self) -> Option<(usize, usize)> {
        // Base address + max register offset
        Some((self.base_register_address, 0x3FFFFFC))
    }
}

#[no_mangle]
pub extern "C" fn plic_handler() {
    let plic = get();

    let source = plic.claim();

    plic.source_handler[source as usize]();

    plic.complete(source);
}
