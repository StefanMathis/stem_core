> **Feedback welcome!**  
> Found a bug, missing docs, or have a feature request?  
> Please open an issue on [GitHub](https://github.com/StefanMathis/stem_core.git).

This crate is part of the stem (Simulation Toolbox for Electric Motors)
framework. See the [stem book] for an introduction.

stem_core builds upon the [stem_magnet] and [stem_slot] crates and provides the
[`LinCore`] and [`RotCore`] types for defining linear and rotary magnetic cores
of radial flux machines. The magnetic core is the centerpiece of the stator or
rotor of an electrical machine which guides the magnetic flux and holds all
other active parts of the motor such as magnets or winding coils. The following
image shows a [`RotCore`] with surface magnets, a double-layered winding and
a [`FluxBarrier`] with interior magnets which has been created with stem_core:

![Showcase of a core with winding zones, interior and surface magnets][full_core_assembly.svg]

_This image was produced with `examples/readme_plots.rs`._

The two fundamental core types [`LinCore`] and [`RotCore`] are modularized: Air
gap shape and flux barriers can be customized independently via [`AirGap`] and
[`FluxBarrier`] (cutouts in the core) trait objects. These traits allow the
usage of user defined air gaps or flux barriers. Additionally, the crate also
provides a couple of predefined [`AirGap`]s and [`FluxBarrier`]s:

![Predefined air gap types][lin_air_gap_comparison.svg]

_From left to right: [`PlainAirGap`], [`SlottedAirGap`] and [`StraightIndentsAirGap`]._

![Predefined flux barrier types][rot_flux_barrier_comparison.svg]

_From left to right: [`Spoke1FluxBarrier`], [`V1rFluxBarrier`] and [`V2rFluxBarrier`]._

# Example

The following code snippet shows how to create the [`RotCore`] showcased in the
first image and how to retrieve various properties from it, which can e.g. be
used for simulations. Note the tight integration of [stem_magnet] and
[stem_slot] into stem_core:

```rust
use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;

use approxim::assert_abs_diff_eq;

use stem_core::prelude::*;
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

let slot: SemiTrapezoidSlot = SemiTrapezoidWidthsAndHeightsBuilder {
    bottom_width: Length::new::<millimeter>(6.76),
    bottom_side_width: Length::new::<millimeter>(6.76),
    top_side_width: Length::new::<millimeter>(8.0),
    top_width: Length::new::<millimeter>(1.5),
    opening_width: Length::new::<millimeter>(1.5),
    bottom_height: Length::new::<millimeter>(0.0),
    side_height: Length::new::<millimeter>(6.79 - 0.75 - 0.5),
    top_height: Length::new::<millimeter>(0.5),
    opening_height: Length::new::<millimeter>(0.75),
    bottom_radius: Length::new::<millimeter>(0.0),
    bottom_side_radius: Length::new::<millimeter>(0.0),
    top_radius: Length::new::<millimeter>(0.0),
    top_side_radius: Length::new::<millimeter>(0.0),
    opening_radius: Length::new::<millimeter>(0.0),
    consider_tooth_tip_leakage: true,
}
.try_into()
.expect("valid slot geometry");

let air_gap = SlottedAirGap::new(28, false, CarterFactorModel::Bin12, Box::new(slot));

let fb = V1rFluxBarrier {
    yoke_distance: Length::new::<millimeter>(4.0),
    relief_path_air_gap_width: Length::new::<millimeter>(5.0),
    relief_path_length: Length::new::<millimeter>(0.0),
    relief_path_width: Length::new::<millimeter>(1.0),
    opening_angle: FRAC_PI_2,
    magnet_space_width: Length::new::<millimeter>(6.0),
    magnet_space_height: Length::new::<millimeter>(23.0),
    glue_gap: Length::new::<millimeter>(0.2),
    leakage_path_width: Length::new::<millimeter>(1.0),
    magnet_material: Some(Arc::new(Material::default())),
    cache: None,
};

let core: RotCore = RotCoreBuilder {
    air_gap_radius: Length::new::<millimeter>(55.0),
    yoke_radius: Length::new::<millimeter>(18.0),
    axial_length: Length::new::<millimeter>(165.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    iron_fill_factor: 1.0,
    material: Arc::new(Material::default()),
    pole_pairs: 2,
    skew_angle: FRAC_PI_2 / 7.0, // One slot pitch
    air_gap: Box::new(air_gap),
    flux_barrier: Some(Box::new(fb)),
}
.try_into().expect("valid core geometry");

// Calculate some properties of the core
assert_abs_diff_eq!(core.volume().get::<cubic_centimeter>(), 962.446, epsilon=1e-3);
assert_abs_diff_eq!(core.mass().get::<kilogram>(), 0.962, epsilon=1e-3);
assert_abs_diff_eq!(core.teeth_mass().get::<kilogram>(), 0.158, epsilon=1e-3);

// Carter factor for a given air gap
assert_abs_diff_eq!(core.carter_factor(Length::new::<millimeter>(0.5)), 1.0478, epsilon=1e-3);

// Evaluate the slotting ordinals (infinite iterator)
use num::rational::Ratio;
let mut ordinals = core.slotting_ordinals();
assert_eq!(ordinals.next(), Some(Ratio::new(14, 1)));
assert_eq!(ordinals.next(), Some(Ratio::new(28, 1)));
assert_eq!(ordinals.next(), Some(Ratio::new(42, 1)));
assert_eq!(ordinals.next(), Some(Ratio::new(56, 1)));
// ...

// Calculate the skew factor for different mechanical ordinals
assert_abs_diff_eq!(core.skew_factor(1), 0.9979, epsilon=1e-3);
assert_abs_diff_eq!(core.skew_factor(7), 0.9003, epsilon=1e-3);

// Iterate over all the orange coils shown in the image (should be 28 * 2 = 56)
assert_eq!(core.winding_zones(&CoilLayout::DoubleHorizontal).count(), 56);

// Return the mass of all interior magnets shown in the image
assert_abs_diff_eq!(core.mass_interior_magnets().get::<kilogram>(), 0.182, epsilon=1e-3);

// Return the mass of all surface magnets shown in the image
let magnet = ArcParallelMagnet::with_const_thickness(
    core.axial_length(),
    core.air_gap_radius(),
    SideHeightOrThickness::Thickness(Length::new::<millimeter>(4.0)),
    AngleOrWidth::Angle(0.5 * FRAC_PI_2 / 2.0),
    Arc::new(Material::default()),
).expect("valid magnet");
let surface_magnets = MagnetAssembly::new(magnet, 1.try_into().expect("not zero"), 2.try_into().expect("not zero"));
assert_abs_diff_eq!(core.mass_surface_magnets(&surface_magnets).get::<kilogram>(), 0.114, epsilon=1e-3);
```

# Serialization and deserialization

If the `serde` feature is enabled, all types from this crate can be
serialized and deserialized. During deserialization, the invariants are
validated (to e.g. prevent a negative axial core length.).

Units and quantities can be deserialized from strings representing SI units via
the [dyn_quantity] crate. Similarily, it is possible to serialize quantities
as value-unit strings using the [serialize_with_units] function.

See the chapter [serialization and deserialization] of the [stem book]
for details.

# Visualization

If the `cairo` feature is enabled, the core shapes can be drawn onto a
[cairo](cairographics.org) context using the drawing mechanics from the
[planar_geo](https://crates.io/crates/planar_geo) crate. See its documentation
for details.

# Acknowledgments

The technical drawings used in the docstrings have been created using 
LibreCAD (<https://librecad.org/>).