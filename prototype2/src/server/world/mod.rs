// Bundles of functions for getting details information into world state
pub mod views;

include!(concat!(env!("OUT_DIR"), "/server/world/mod.rs"));
