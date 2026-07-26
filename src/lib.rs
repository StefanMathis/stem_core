pub mod error;
pub mod magnets;
pub mod winding_zones;

pub mod air_gap;
pub mod core;
pub mod flux_barrier;

pub use stem_magnet;
pub use stem_slot;
pub use stem_slot::planar_geo;

pub enum LinOrRot {
    Lin,
    Rot,
}

#[cfg(feature = "cairo")]
pub const GRAY: planar_geo::draw::Color = planar_geo::draw::Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

pub mod prelude {
    pub use crate::magnets::*;
    pub use crate::winding_zones::*;

    pub use crate::air_gap::*;
    pub use crate::core::*;
    pub use crate::flux_barrier::*;

    #[doc(hidden)]
    pub use stem_magnet;
    #[doc(hidden)]
    pub use stem_slot;

    #[doc(hidden)]
    pub use stem_magnet::arc::*;
    #[doc(hidden)]
    pub use stem_magnet::assembly::MagnetAssembly;
    #[doc(hidden)]
    pub use stem_magnet::block::BlockMagnet;
    #[doc(hidden)]
    pub use stem_magnet::bread_loaf::BreadLoafMagnet;
    #[doc(hidden)]
    pub use stem_magnet::magnet::Magnet;

    // Prevent rustdoc from documenting
    #[doc(hidden)]
    pub use stem_slot::prelude::*;
}
