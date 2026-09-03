/*!
[serialize_with_units]: https://docs.rs/dyn_quantity/latest/dyn_quantity/quantity/serde_impl/fn.serialize_with_units.html
[dyn_quantity]: https://crates.io/crates/dyn_quantity
[serialization and deserialization]: https://stefanmathis.github.io/stem_book/serialization_and_deserialization.html
[stem book]: https://stefanmathis.github.io/stem_book/
[stem_magnet]: https://crates.io/crates/stem_magnet
[stem_slot]: https://crates.io/crates/stem_slot
[`LinCore`]: crate::core::LinCore
[`RotCore`]: crate::core::RotCore
[`AirGap`]: crate::air_gap::AirGap
[`PlainAirGap`]: crate::air_gap::PlainAirGap
[`SlottedAirGap`]: crate::air_gap::SlottedAirGap
[`StraightIndentsAirGap`]: crate::air_gap::StraightIndentsAirGap
[`FluxBarrier`]: crate::flux_barrier::FluxBarrier
[`Spoke1FluxBarrier`]: crate::flux_barrier::Spoke1FluxBarrier
[`V1rFluxBarrier`]: crate::flux_barrier::V1rFluxBarrier
[`V2rFluxBarrier`]: crate::flux_barrier::V2rFluxBarrier

Magnetic core definition for stem - a Simulation Toolbox for Electric Motors.

 */
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("full_core_assembly.svg", "docs/img/full_core_assembly.svg"),
doc = ::embed_doc_image::embed_image!("lin_air_gap_comparison.svg", "docs/img/lin_air_gap_comparison.svg"),
doc = ::embed_doc_image::embed_image!("rot_flux_barrier_comparison.svg", "docs/img/rot_flux_barrier_comparison.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
#![doc = include_str!("../docs/main.md")]
#![deny(missing_docs)]

pub mod error;
pub mod magnets;
pub mod winding_zones;

pub mod air_gap;
pub mod core;
pub mod flux_barrier;

pub use stem_magnet;
pub use stem_magnet::stem_material;
pub use stem_slot;
pub use stem_slot::planar_geo;
pub use stem_slot::stem_coil_layout;

/// An indicator of whether an entity is linear or rotary.
///
/// This enum is created from functions such as
/// [`CoreExt::lin_or_rot`](crate::core::CoreExt::lin_or_rot) and states if the
/// entity (in this case a magnetic core) which created it is linear or rotary.
pub enum LinOrRot {
    /// Linear entity.
    Lin,
    /// Rotary entity.
    Rot,
}

/**
Standard [`Color`](planar_geo::draw::Color) for drawing magnetic cores.

This color is used as the
[`Style::background_color`](planar_geo::draw::Style::background_color)s of the
[`DrawableRef`](planar_geo::draw::DrawableRef)s returned by
[`CoreExt::drawable`](crate::core::CoreExt::drawable). All the core images
thorough this create use this color.
 */
#[cfg(feature = "cairo")]
pub const GRAY: planar_geo::draw::Color = planar_geo::draw::Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

pub mod prelude {
    /*!
    This module reexports the core, air gap and flux barrier types defined in
    this crate as well as the [`Magnets`] and [`WindingZones`] iterators. In
    addition, it reexports the prelude modules of the [`stem_slot`] and
    [`stem_magnet`] dependencies (and therefore also [`stem_material::prelude`]).
     */

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

    #[doc(hidden)]
    pub use stem_material;

    // Prevent rustdoc from documenting
    #[doc(hidden)]
    pub use stem_slot::prelude::*;
}
