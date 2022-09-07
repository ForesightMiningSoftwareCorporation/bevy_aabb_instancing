//! [![crates.io](https://img.shields.io/crates/v/bevy-aabb-instancing)](https://crates.io/crates/bevy-aabb-instancing)
//! [![docs.rs](https://docs.rs/bevy-aabb-instancing/badge.svg)](https://docs.rs/bevy-aabb-instancing)
//!
//! Render millions of AABBs every frame with an instancing renderer.
//!
//! ![Example](https://raw.githubusercontent.com/ForesightMiningSoftwareCorporation/bevy-aabb-instancing/main/examples/wave.png)
//!
//! # Demo
//!
//! ```sh
//! cargo run --example wave --release
//! ```
//!
//! # Features
//!
//! - vertex pulling renderer
//! - cuboid outlining
//! - clipping planes
//! - depth jitter to counteract z-fighting of coplanar cuboids
//!
//! # License
//!
//! Licensed under the Apache License Version 2.0 by copyright holders Duncan
//! Fairbanks and Foresight Mining Software Corporation.
//!
//! ## Sponsors
//!
//! The creation and maintenance of `bevy_aabb_instancing` is sponsored by
//! Foresight Mining Software Corporation.
//!
//! <img
//! src="https://user-images.githubusercontent.com/2632925/151242316-db3455d1-4934-4374-8369-1818daf512dd.png"
//! alt="Foresight Mining Software Corporation" width="480">

mod component;
mod vertex_pulling;

pub use component::*;
pub use vertex_pulling::plugin::*;

type SmallKeyHashMap<K, V> = ahash::AHashMap<K, V>;
