extern crate opencv;
extern crate libc;
use opencv::core;

#[test]
fn test_primitives() {
    let ticks = core::get_tick_count().unwrap();
    assert!(ticks > 10000);
    let freq = core::get_tick_frequency().unwrap();
    assert!(freq > 1000f64);
    let cpus = core::get_number_of_cp_us().unwrap();
    assert!(cpus > 0);
    core::set_use_optimized(true).unwrap();
    let optims = core::use_optimized().unwrap();
    assert!(optims);
    core::set_use_optimized(false).unwrap();
    let optims = core::use_optimized().unwrap();
    assert!(!optims);
}

#[test]
fn test_optimized() {
    core::set_use_optimized(true).unwrap();
    let optims = core::use_optimized().unwrap();
    assert!(optims);
}

#[test]
fn test_optimized2() {
    core::set_use_optimized(true).unwrap();
    let optims = core::use_optimized().unwrap();
    assert!(optims);
    core::set_use_optimized(false).unwrap();
    let optims = core::use_optimized().unwrap();
    assert!(!optims);
}

#[test]
fn test_unoptimized() {
    core::set_use_optimized(false).unwrap();
    let optims = core::use_optimized().unwrap();
    assert!(!optims);
}

#[test]
fn test_cpus() {
    let cpus = core::get_number_of_cp_us().unwrap();
    assert!(cpus > 0);
}

#[test]
fn test_ticks() {
    let ticks = core::get_tick_count().unwrap();
    assert!(ticks > 10000);
}

#[test]
fn test_freq() {
    let freq = core::get_tick_frequency().unwrap();
    assert!(freq > 1000f64);
}

#[test]
fn test_return_string() {
    let info = core::get_build_information().unwrap();
    assert!(info.contains("\nGeneral configuration for OpenCV"));
}
