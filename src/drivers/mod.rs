#[cfg(drv_ns16550)]
pub mod ns16550;

#[cfg(drv_plic)]
pub mod plic;

trait Driver {
    fn probe(compat: &str) -> bool;
}

pub static DRIVERS_PROBE: &[fn(&str) -> bool] = &[ns16550::Ns16550::probe];

// TODO: - have a driver trait
//       - build a list of all the drivers we have compiled, list would countain `impl Driver`
//         stuff, or a functions pointers to instantiate drivers.
//         like a list of struct CompiledDriver {
//                            name: &'static
//                            driver: Fn(struct Driver &) -> impl Driver,
