/*!
This module contains the [`CoreExt`] trait, which provides shared functionality
for all core types: [`LinCore`](crate::core::LinCore),
[`RotCore`](crate::core::RotCore), and the [`Core`](crate::core::Core) and
[`CoreRef`] enums. It is a sealed trait. See its docstring for more.
*/

use std::sync::Arc;

use planar_geo::prelude::*;
use rayon::prelude::*;
use stem_magnet::prelude::*;
use stem_slot::{
    coil_layout::CoilLayout, current_displacement::CurrentDisplacementCalculator, slot::Slot,
};

use super::CoreRef;
use crate::{
    LinOrRot,
    air_gap::AirGap,
    flux_barrier::FluxBarrier,
    magnets::Magnets,
    winding_zones::{PositionedZoneContour, WindingZones},
};

pub(crate) mod private {
    /// Sealed trait for [`CoreExt`]
    pub trait Sealed {}
}

/**
When a [`assembly_check`](CoreExt::assembly_check) fails, this enum describes
one of the components which collided with another component. See
[`AssemblyFailure`] for more information.
 */
#[derive(Clone, Debug)]
pub enum Component {
    /// Shape of the core  which collided with another component.
    Core(Shape),
    /// Contour of a winding zone which collided with another component.
    Zone {
        /// The nth element of the [`WindingZones`] iterator created by
        /// [`CoreExt::winding_zones`], which collided with another component.
        idx: usize,
        /// The winding zone contour and the
        /// [`Zone`](crate::winding_zones::Zone) index.
        contour: PositionedZoneContour,
    },
    /// Shape of a surface magnet which collided with another component.
    SurfaceMagnet {
        /// The nth element of the [`Magnets`] iterator created by
        /// [`CoreExt::surface_magnets`], which collided with another component.
        idx: usize,
        /// The shape of the colliding surface magnet.
        shape: Shape,
    },
    /// Shape of an interior magnet which collided with another component.
    InteriorMagnet {
        /// The nth element of the [`Magnets`] iterator created by
        /// [`CoreExt::interior_magnets`], which collided with another
        /// component.
        idx: usize,
        /// The shape of the colliding interior magnet.
        shape: Shape,
    },
}

/**
An error type created by [`CoreExt::assembly_check`] which describes a
collision between two components of an active part (magnetic core,
winding zones, surface magnets, interior magnets). It holds the two colliding
components and the reason for the collision.
 */
#[derive(Clone, Debug)]
pub struct AssemblyFailure {
    /// One of the colliding components.
    pub left_component: Component,
    /// The component which collided with the `left_component`.
    pub right_component: Component,
    /// Reason for the collision.
    pub reason: AssemblyFailureReason,
}

/**
An enum which describes the reason for a collision between the two components of
an active part. It is created as part of the [`AssemblyFailure`] error when a
[`assembly_check`](CoreExt::assembly_check) fails.
 */
#[derive(Clone, Debug)]
pub enum AssemblyFailureReason {
    /// The two components are overlapping.
    Overlap(Overlap),
    /// One component which should be contained by another one isn't. An example
    /// would be an interior magnet which is not contained by the
    /// [`CoreExt::shape`].
    NotContained(NotContained),
}

impl From<Overlap> for AssemblyFailureReason {
    fn from(value: Overlap) -> Self {
        Self::Overlap(value)
    }
}

impl From<NotContained> for AssemblyFailureReason {
    fn from(value: NotContained) -> Self {
        Self::NotContained(value)
    }
}

/**
A sealed trait providing shared functionality for all core types:
[`LinCore`](crate::core::LinCore), [`RotCore`](crate::core::RotCore), and the
[`Core`](crate::core::Core) and [`CoreRef`] enums.

The main purpose of this enum is to provide a common interface for all core
types, allowing for polymorphic behavior and code reuse. It is sealed to prevent
external implementations, ensuring that only the intended core types can
implement it.

Some of the trait methods require different implementations depending on the
[`AirGap`] or [`FluxBarrier`] of the core. These methods are wrappers around
the methods of [`AirGap`] / [`FluxBarrier`]; an example would be
[`CoreExt::winding_zones`] which wraps [`AirGap::winding_zones`]. The wrappers
always have the signature `wrapper(core, ...)`, while the wrappers look like
this: `wrapped(air_gap / flux_barrier, core, ...)`. Using the example of the
`winding_zones` method, the wrapper implementation looks like this:

```ignore
fn winding_zones(&self, coil_layout: &CoilLayout) -> WindingZones {
    return self
        .air_gap()
        .winding_zones(self.as_core_ref(), coil_layout);
}
```

The wrapped methods are used as way to implement polymorphism and are not
intended to be used standalone. In particular, calling them with a `core` which
wasn't build using the specified [`AirGap`] / [`FluxBarrier`] trait object may
result in incorrect or unexpected results (although it must never result in
undefined behaviour)! The implementation of the [`AirGap`] / [`FluxBarrier`]
traits is not required to guard against this misuse of the interface.
*/
pub trait CoreExt: Sync + Send + std::fmt::Debug + private::Sealed {
    /// Converts the reference of `self` into a [`CoreRef`] enum.
    ///
    /// This method is used for the [`AirGap`] and [`FluxBarrier`] traits to
    /// access the core's properties without requiring generics, because those
    /// traits need to be object-safe. The [`CoreRef`] enum acts as a
    /// type-erased wrapper around the core, allowing for dynamic dispatch
    /// and polymorphism.
    fn as_core_ref(&self) -> CoreRef<'_>;

    /// Returns a reference to the [`AirGap`] trait object describing the air
    /// gap contour of the core.
    fn air_gap(&self) -> &dyn AirGap;

    /// Returns a reference to the [`FluxBarrier`] trait object describing the
    /// flux barrier of the core, if it has one. If it hasn't, this method
    /// returns `None`.
    fn flux_barrier(&self) -> Option<&dyn FluxBarrier>;

    /// Returns the axial length of the core, i.e. the length into the image
    /// plane when looking at the cross section of the core.
    ///
    /// See the docstrings of [`LinCore`](crate::core::LinCore) and
    /// [`RotCore`](crate::core::RotCore) for a visualization.
    fn axial_length(&self) -> Length;

    /// Returns the number of pole pairs of `self`.
    fn pole_pairs(&self) -> u16;

    /// Returns a reference to the core [`Material`].
    fn material(&self) -> &Arc<Material>;

    /// Returns the iron fill factor of the core, which is the ratio of the iron
    /// volume to the total core volume.
    ///
    /// Magnetic cores are often made from laminated steel sheets which are
    /// glued together in order to reduce eddy current losses. This reduces the
    /// effective iron volume and hence the magnetic permeability of the core.
    /// This can be accounted for by using a fictive [`CoreExt::iron_length`],
    /// which is the product of the [`CoreExt::axial_length`] and the iron fill
    /// factor. The iron fill factor is the ratio between the steel and the
    /// total lamination thickness as shown in the image below. Typical values
    /// are between 0.9 and 1, depending on the glue thickness.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Iron fill factor][cad_iron_fill_factor]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_iron_fill_factor",
            "docs/img/cad_iron_fill_factor.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    fn iron_fill_factor(&self) -> f64 {
        // embed_doc_image does not support documenting trait methods without
        // a body, so a dummy implementation is provided (which is overridden
        // by the actual implementors).
        return 0.0;
    }

    /// Returns the yoke height of the core.
    ///
    /// The yoke is the "backbone" of the core, where the magnetic flux lines
    /// coming from the air gap closes. Its height is the
    /// [`LinCore::height`](crate::core::LinCore::height) or the absolute
    /// difference between
    /// [`RotCore::yoke_radius`](crate::core::RotCore::yoke_radius) and
    /// [`RotCore::air_gap_radius`](crate::core::RotCore::air_gap_radius) minus
    /// the [`CoreExt::tooth_height`]. The following image shows the dimensions
    /// for a [`PlainAirGap`](crate::air_gap::PlainAirGap) and a
    /// [`SlottedAirGap`](crate::air_gap::SlottedAirGap):
    #[doc = ""]
    #[cfg_attr(feature = "doc-images", doc = "![Yoke height][cad_yoke_tooth_height]")]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_yoke_tooth_height",
            "docs/img/cad_yoke_tooth_height.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    fn yoke_height(&self) -> Length {
        // embed_doc_image does not support documenting trait methods without
        // a body, so a dummy implementation is provided (which is overridden
        // by the actual implementors).
        return Length::new::<meter>(0.0);
    }

    /// Returns the skew angle of the core.
    ///
    /// The magnetic field in the air gap is composed of various harmonics,
    /// which result from both the magnet field sources (windings and magnets)
    /// and the core geometry, particularly the air gap contour. For example,
    /// when a core is slotted, the resulting field will have "slot harmonics"
    /// whose order is a multiple of [`CoreExt::slots`]. With the exception of
    /// the pole pair (main) harmonic, which provides the continouos force
    /// driving the motor, these harmonics should be usually minimized because
    /// they result in force fluctations and noise.
    ///
    /// One way to suppress the harmonics is to "skew" the motor by the
    /// so-called skew angle. In the case of a linear motor, this means shifting
    /// one end by `skew_angle / (2*pi) * core.width()` against the other. A
    /// rotary motor is twisted along its rotational axis by the skew angle.
    /// Depending on the angle, this leads to destructive interference of
    /// undesired harmonics. For example, to suppress the aforementioned slot
    /// harmonics, the core needs to be skewed by `2 * pi / core.slots()`. See
    /// [\[1\]](#core_ext_skew_angle_1), section 6.5 for more. Be aware that
    /// skewing affects all harmonics, including the useful (force-producing)
    /// first stator harmonic. The general goal is therefore to find an angle
    /// which suppresses the unwanted harmonics while reducing the useful ones
    /// as little as possible.
    ///
    /// The exact realization of the skewing depends on
    /// [`CoreExt::num_segments`]. If this number is zero, the core is skewed
    /// continuously along its axial length. For the typical case of a laminated
    /// core, each lamination sheet is shifted a bit against its neighbors.
    /// Otherwise, the core is discretized ("staggered") into the specified
    /// number of segments which are shifted by `skew_angle / num_segments`
    /// against each other. The amount of segments heavily influences which
    /// harmonics are suppressed, see the docstring of [`skew_factor`] for
    /// details.
    ///
    /// # Literature
    /// <a id="core_ext_skew_angle_1">\[1\]</a>
    /// Binder, Andreas: Elektrische Maschinen und Antriebe (2012), Springer-
    /// Verlag, Berlin Heidelberg
    fn skew_angle(&self) -> f64;

    /**
    Returns the air gap "width" of the core.

    For a linear core, this is [`LinCore::width`](crate::core::LinCore::width).
    For a rotary core, this is the air gap outline
    ([`RotCore::air_gap_radius`](crate::core::RotCore::air_gap_radius) times 2
    pi).

    # Examples

    ```
    use std::sync::Arc;
    use std::f64::consts::TAU;
    use stem_core::prelude::*;

    let lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(100.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: None,
    }.try_into().expect("valid inputs");
    assert_eq!(lin_core.air_gap_width().get::<millimeter>(), 100.0);

    let rot_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(50.0),
        yoke_radius: Length::new::<millimeter>(90.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: None,
    }.try_into().expect("valid inputs");
    assert_eq!(rot_core.air_gap_width().get::<millimeter>(), 50.0 * TAU);
    ```
     */
    fn air_gap_width(&self) -> Length;

    /// Returns if the core is linear or rotary.
    fn lin_or_rot(&self) -> LinOrRot;

    /**
    Returns the total axial coil overhang on boths sides of the magnetic core.

    If the core holds a winding, its coils can generally be separated into two
    different parts: A straight one where the coil goes along the axial length
    of the core and the end winding, where the coil closes. Sometimes, the
    straight part extends a bit from the core ends due to e.g. the slot
    insulation extending outside the core. This "extended" coil length is called
    the axial coil overhang. The ASCII art below visualizes it with equal signs
    (=). If = equals one mm, the return value of `axial_coil_overhang` would be
    3 mm.
    ```ignore
         ┌──────┐
    ┌──==│      │=──┐
    │    │ Core │   │ <-- Coil
    └──==│      │=──┘
         └──────┘
    ```

    The total length of a full turn is therefore
    `2 * (axial_length + axial_coil_overhang + end_winding_half_turn_length)`.
    Since this part of the coil is outside the core, it is not considered when
    calculating core-dependent parameters such as the main inductance. Instead,
    it is treated as part of the end winding and therefore is added to the
    `end_winding_half_turn_length` when calculating parameters such as the end
    winding resistance or inductance.
     */
    fn axial_coil_overhang(&self) -> Length;

    /// Returns a reference to the cross-section shape of `self`.
    fn shape(&self) -> &Shape;

    /// Returns the d-axis pole coverage of `self` as a relative fraction of the
    /// entire air gap surface (value betwen 0 and 1). 0 means that the entire
    /// air gap surface is covered by the q-axis, 1 means that it is completely
    /// covered by the d-axis.
    ///
    /// If the core holds no magnets and has no flux barrier, this value is
    /// simply 0.5 (air gap evenly split between d- and q-axis). If a surface
    /// magnet assembly is provided as the second argument, the area covered by
    /// magnets is divided by the entire air gap area to get the pole
    /// coverage. If no surface magnet assembly is given, but the core holds a
    /// flux barrier, this methods forwards to [`FluxBarrier::pole_coverage`].
    ///
    /// The image below all three of these cases, using a
    /// [`V1rFluxBarrier`](crate::flux_barrier::V1rFluxBarrier) for the
    /// rightmost core, where the
    /// d-axis covers 4 of 7 teeth per pole and hence the resulting pole
    /// coverage is 4/7 ≈ 0.571.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Pole coverage comparison][pole_coverage]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image("pole_coverage", "docs/img/pole_coverage.svg")
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// # Examples
    ///
    /// This example calculates the pole coverages for the three core
    /// configurations shown in the image above.
    ///
    /// ```
    /// use std::f64::consts::FRAC_PI_2;
    /// use std::sync::Arc;
    ///
    /// use approx::assert_abs_diff_eq;
    ///
    /// use stem_core::prelude::*;
    /// use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;
    ///
    /// let slot: SemiTrapezoidSlot = SemiTrapezoidWidthsAndHeightsBuilder {
    ///     bottom_width: Length::new::<millimeter>(6.76),
    ///     bottom_side_width: Length::new::<millimeter>(6.76),
    ///     top_side_width: Length::new::<millimeter>(8.0),
    ///     top_width: Length::new::<millimeter>(1.5),
    ///     opening_width: Length::new::<millimeter>(1.5),
    ///     bottom_height: Length::new::<millimeter>(0.0),
    ///     side_height: Length::new::<millimeter>(6.79 - 0.75 - 0.5),
    ///     top_height: Length::new::<millimeter>(0.5),
    ///     opening_height: Length::new::<millimeter>(0.75),
    ///     bottom_radius: Length::new::<millimeter>(0.0),
    ///     bottom_side_radius: Length::new::<millimeter>(0.0),
    ///     top_radius: Length::new::<millimeter>(0.0),
    ///     top_side_radius: Length::new::<millimeter>(0.0),
    ///     opening_radius: Length::new::<millimeter>(0.0),
    ///     consider_tooth_tip_leakage: true,
    /// }
    /// .try_into().expect("valid slot");
    ///
    /// let air_gap = SlottedAirGap::new(28, false, CarterFactorModel::Bin12, Box::new(slot));
    /// let mut core: RotCore = RotCoreBuilder {
    ///     air_gap_radius: Length::new::<millimeter>(54.4),
    ///     yoke_radius: Length::new::<millimeter>(19.0),
    ///     axial_length: Length::new::<millimeter>(165.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 2,
    ///     skew_angle: 0.0,
    ///     air_gap: Box::new(air_gap),
    ///     flux_barrier: None,
    /// }
    /// .try_into().expect("valid core");
    ///
    /// // Core without surface magnets or surface assembly
    /// assert_eq!(core.pole_coverage(None), 0.5);
    ///
    /// // With surface magnets
    /// let angle = 0.35 * FRAC_PI_2; // One pole is one quarter arc, 2 magnets per pole -> 0.7 pole coverage
    /// let magnet = ArcParallelMagnet::with_const_thickness(
    ///     core.axial_length(),
    ///     core.air_gap_radius(),
    ///     SideHeightOrThickness::Thickness(Length::new::<millimeter>(4.0)),
    ///     AngleOrWidth::Angle(angle),
    ///     Arc::new(Material::default()),
    /// ).expect("valid magnet");
    /// let mag_assembly = MagnetAssembly::new(magnet, 1.try_into().expect("not zero"), 2.try_into().expect("not zero"));
    ///
    /// assert_abs_diff_eq!(core.pole_coverage(Some(&mag_assembly)), 0.7, epsilon = 1e-4);
    ///
    /// // With flux barrier
    /// let barrier = V1rFluxBarrier {
    ///     yoke_distance: Length::new::<millimeter>(4.5),
    ///     relief_path_air_gap_width: Length::new::<millimeter>(3.5),
    ///     relief_path_length: Length::new::<millimeter>(1.33),
    ///     relief_path_width: Length::new::<millimeter>(4.0),
    ///     opening_angle: FRAC_PI_2,
    ///     magnet_space_width: Length::new::<millimeter>(10.0),
    ///     magnet_space_height: Length::new::<millimeter>(20.0),
    ///     glue_gap: Length::new::<millimeter>(0.2),
    ///     leakage_path_width: Length::new::<millimeter>(1.0),
    ///     magnet_material: Some(Arc::new(Material::default())),
    ///     cache: None,
    /// };
    /// core.set_flux_barrier(Some(Box::new(barrier))).expect("compatible to core");
    ///
    /// assert_abs_diff_eq!(core.pole_coverage(None), 4.0 / 7.0, epsilon = 1e-4);
    /// ```
    fn pole_coverage(&self, _surface_magnet_assembly: Option<&MagnetAssembly>) -> f64 {
        // embed_doc_image does not support documenting trait methods without
        // a body, so a dummy implementation is provided (which is overridden
        // by the actual implementors).
        return 0.5;
    }

    // =========================================================================

    /// Returns the total number of poles.
    ///
    /// This is 2 * [`Self::pole_pairs`].
    fn poles(&self) -> u16 {
        return 2 * self.pole_pairs();
    }

    /**
    Checks if the core can be assembled with the given `coil_layout` and
    `surface_magnet_assembly`, using `epsilon` and `max_relative` as tolerances
    for overlap checks.

    This method first collects all geometric bodies ([`Shape`]s and
    [`Contour`]s) belonging to the assembly:
    - The core shape itself ([`CoreExt::shape`]),
    - The [`PositionedZoneContour`]s returned by the [`CoreExt::winding_zones`]
    method, using `coil_layout` as the second argument (if the core is not
    windable, the iterator will return `None`),
    - If a `surface_magnet_assembly` was provided, the magnet shapes created via
    [`CoreExt::surface_magnets`],
    - If the core has a [`flux_barrier`](`CoreExt::flux_barrier`) which can hold
    magnets, their shapes returned by [`CoreExt::interior_magnets`].

    If any two of these bodies overlap (checked with
    [`Composite::contains_any_composite`] using the provided `epsilon` and
    `max_relative` tolerances, an [`AssemblyFailure`] is returned, containing
    both the two overlapping bodes as well as the exact [`Overlap`] provided by
    [`Composite::contains_any_composite`]. Furthermore, all interior magnets of
    should be contained within the [`Shape::contour`] of the core shape, which
    is tested with [`Composite::contains_composite`]. If this is not the case,
    the core shape and the magnet shape are returned together with a
    [`NotContained`] enum providing more details.

    These checks are performed concurrently. If multiple issues exist, the one
    found first will be returned. Hence, this method may return different
    [`AssemblyFailure`]s when called repeatedly.

    # Examples

    The following example shows a successfull and a failing assembly check for
    two different surface magnet assemblies. In the failing case, the magnet
    covers a full quarter circle, which leads to an overlap due to the core
    having 6 magnets.

    ```
    use std::f64::consts::FRAC_PI_2;
    use std::sync::Arc;

    use stem_core::prelude::*;
    use stem_core::planar_geo::prelude::*;

    let air_gap_plain = PlainAirGap::default();

    let core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(53.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap_plain),
        flux_barrier: None,
    }
    .try_into()
    .unwrap();

    // Magnet covers one eight of a circle (0.5 * FRAC_PI_2)
    let magnet = ArcParallelMagnet::with_const_thickness(
        core.axial_length(),
        core.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(4.0)),
        AngleOrWidth::Angle(0.5 * FRAC_PI_2),
        Arc::new(Material::default()),
    ).expect("valid magnet");
    let assembly_1 = MagnetAssembly::new(magnet, 1.try_into().expect("not zero"), 1.try_into().expect("not zero"));

    assert!(core.assembly_check(&CoilLayout::SingleFilled, Some(&assembly_1), DEFAULT_EPSILON, DEFAULT_MAX_RELATIVE).is_ok());

    // Magnet covers one quarter of a circle (FRAC_PI_2)
    let magnet = ArcParallelMagnet::with_const_thickness(
        core.axial_length(),
        core.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(4.0)),
        AngleOrWidth::Angle(FRAC_PI_2),
        Arc::new(Material::default()),
    ).expect("valid magnet");
    let assembly_2 = MagnetAssembly::new(magnet, 1.try_into().expect("not zero"), 1.try_into().expect("not zero"));

    assert!(core.assembly_check(&CoilLayout::SingleFilled, Some(&assembly_2), DEFAULT_EPSILON, DEFAULT_MAX_RELATIVE).is_err());
    ```
     */
    fn assembly_check(
        &self,
        coil_layout: &CoilLayout,
        surface_magnet_assembly: Option<&MagnetAssembly>,
        epsilon: f64,
        max_relative: f64,
    ) -> Result<(), AssemblyFailure> {
        let core_shape = self.shape();
        let zones: Vec<PositionedZoneContour> = self.winding_zones(coil_layout).collect();
        if let Some(o) = zones.par_iter().enumerate().find_map_any(|(i1, z1)| {
            if let Ok(overlap) =
                core_shape.contains_any_composite(&z1.contour, epsilon, max_relative)
            {
                return Some(AssemblyFailure {
                    left_component: Component::Core(core_shape.clone()),
                    right_component: Component::Zone {
                        idx: i1,
                        contour: z1.clone(),
                    },
                    reason: overlap.into(),
                });
            }

            if let Some(o) = zones.par_iter().enumerate().find_map_any(|(i2, z2)| {
                if i1 <= i2 {
                    return None;
                }
                if let Ok(overlap) =
                    z2.contour
                        .contains_any_composite(&z1.contour, epsilon, max_relative)
                {
                    return Some(AssemblyFailure {
                        left_component: Component::Zone {
                            idx: i1,
                            contour: z1.clone(),
                        },
                        right_component: Component::Zone {
                            idx: i2,
                            contour: z2.clone(),
                        },
                        reason: overlap.into(),
                    });
                }
                return None;
            }) {
                return Some(o);
            }

            return None;
        }) {
            return Err(o);
        }

        // Check surface magnets
        if let Some(magnets) = surface_magnet_assembly.map(|m| {
            self.surface_magnets(m, false)
                .map(|e| e.shape)
                .collect::<Vec<_>>()
        }) {
            if let Some(o) = magnets.par_iter().enumerate().find_map_any(|(i1, m1)| {
                if let Ok(overlap) = core_shape.contains_any_composite(m1, epsilon, max_relative) {
                    return Some(AssemblyFailure {
                        left_component: Component::Core(core_shape.clone()),
                        right_component: Component::SurfaceMagnet {
                            idx: i1,
                            shape: m1.clone(),
                        },
                        reason: overlap.into(),
                    });
                }

                if let Some(o) = magnets.par_iter().enumerate().find_map_any(|(i2, m2)| {
                    if i1 <= i2 {
                        return None;
                    }

                    if let Ok(overlap) = m2.contains_any_composite(m1, epsilon, max_relative) {
                        return Some(AssemblyFailure {
                            left_component: Component::SurfaceMagnet {
                                idx: i1,
                                shape: m1.clone(),
                            },
                            right_component: Component::SurfaceMagnet {
                                idx: i2,
                                shape: m2.clone(),
                            },
                            reason: overlap.into(),
                        });
                    }
                    return None;
                }) {
                    return Some(o);
                }

                if let Some(o) = zones.par_iter().enumerate().find_map_any(|(i, z)| {
                    if let Ok(overlap) =
                        m1.contains_any_composite(&z.contour, epsilon, max_relative)
                    {
                        return Some(AssemblyFailure {
                            left_component: Component::Zone {
                                idx: i,
                                contour: z.clone(),
                            },
                            right_component: Component::SurfaceMagnet {
                                idx: i1,
                                shape: m1.clone(),
                            },
                            reason: overlap.into(),
                        });
                    }
                    return None;
                }) {
                    return Some(o);
                }

                return None;
            }) {
                return Err(o);
            }
        }

        // Check interior magnets
        // Condition A: Must be inside core contour
        // Condition B: Must not overlap core contour
        let interior_magnets: Vec<_> = self.interior_magnets(false).collect();
        if let Some(o) = interior_magnets
            .par_iter()
            .enumerate()
            .find_map_any(|(i, m)| {
                if let Err(e) = core_shape
                    .contour()
                    .contains_shape(&m.shape, epsilon, max_relative)
                {
                    return Some(AssemblyFailure {
                        left_component: Component::Core(core_shape.clone()),
                        right_component: Component::InteriorMagnet {
                            idx: i,
                            shape: m.shape.clone(),
                        },
                        reason: e.into(),
                    });
                }

                if let Ok(overlap) = core_shape.contains_any_shape(&m.shape, epsilon, max_relative)
                {
                    return Some(AssemblyFailure {
                        left_component: Component::Core(core_shape.clone()),
                        right_component: Component::InteriorMagnet {
                            idx: i,
                            shape: m.shape.clone(),
                        },
                        reason: overlap.into(),
                    });
                }

                return None;
            })
        {
            return Err(o);
        }

        return Ok(());
    }

    /**
    Returns the slot pitch of the core. This is the quotient of the
    [`air_gap_width`](CoreExt::air_gap_width) and the number of
    [`slots`](CoreExt::slots).
     */
    fn slot_pitch(&self) -> Length {
        self.air_gap_width() / self.slots() as f64
    }

    /**
    Returns the carter factor of `self`.

    The _carter factor_ `kc `describes the effect of non-smooth (e.g. slotted)
    air gaps contours on the magnetic resistance / reluctance of the air gap.
    The magnetically effective air gap width can be calculated as
    `kc_stator_core * kc_rotor_core * geometric_air_gap_width` with both factors
    being equal to or larger than 1.

    The exact implementation of the carter factor calculation depends on the
    [`AirGap`] itself, hence this method forwards to [`AirGap::carter_factor`],
    using `self` as the second argument and `air_gap_width` as the third. See
    the docstring of [`AirGap::carter_factor`] for details and examples.
     */
    fn carter_factor(&self, air_gap_width: Length) -> f64 {
        self.air_gap()
            .carter_factor(self.as_core_ref(), air_gap_width)
    }

    /// Returns the discretization / number of segments of the core.
    ///
    /// Depending on its [`AirGap`], a core may be composed of multiple
    /// individual segments  against each other as defined by the
    /// [`CoreExt::skew_angle`]. This affects the [`skew_factor`] of the core,
    /// which can be used to suppress unwanted magnetic harmonics. See the
    /// docstrings of [`CoreExt::skew_angle`] and [`skew_factor`] for details.
    /// If this value is zero, the core is continuously skewed. If it is one,
    /// the component is not skewed at all, as it consists of a single straight
    /// (non-twisted) segment.
    ///
    /// This method forwards to [`AirGap::num_segments`], using `self` as the
    /// second argument.
    fn num_segments(&self) -> usize {
        return self.air_gap().num_segments(self.as_core_ref());
    }

    /// Returns an iterator over the [`PositionedZoneContour`]s for the given
    /// `coil_layout`.
    ///
    /// If a core [`is_windable`](CoreExt::is_windable), this iterator returns
    /// the contours of all of its winding zones. The winding zone contours
    /// depend on the [`AirGap`] of the core: For example, the winding zones of
    /// a [`SlottedAirGap`](crate::air_gap::SlottedAirGap) are inside its slots,
    /// whereas those of a [`PlainAirGap`](crate::air_gap::PlainAirGap) are
    /// located on the top of the air gap contour / inside the air gap itself.
    /// The image below shows the contours and their return order for the
    /// aforementioned examples of a slotted and a plain air gap with
    /// a [`CoilLayout::DoubleHorizontal`].
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Winding zones for a slotted and a plain air gap][winding_zones]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image("winding_zones", "docs/img/winding_zones.svg")
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    /// This method forwards to [`AirGap::winding_zones`], using `self` as the
    /// second and `coil_layout` as the third argument.
    ///
    /// # Examples
    ///
    /// The number of winding zones is equal to the number of slots times the
    /// number of [`CoilLayout::layers`]:
    ///
    /// ```
    /// use std::f64::consts::PI;
    /// use std::sync::Arc;
    ///
    /// use approx::assert_abs_diff_eq;
    ///
    /// use stem_core::prelude::*;
    /// use stem_slot::semi_trapezoid::SemiTrapezoidWithoutSlopesBuilder;
    ///
    /// let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
    ///     bottom_width: Length::new::<millimeter>(9.0),
    ///     opening_width: Length::new::<millimeter>(2.0),
    ///     height: Length::new::<millimeter>(20.0),
    ///     opening_height: Length::new::<millimeter>(2.0),
    ///     slot_angle: 10.0 * PI / 180.0,
    ///     bottom_radius: Length::new::<millimeter>(2.0),
    ///     top_radius: Length::new::<millimeter>(1.0),
    ///     opening_radius: Length::new::<millimeter>(0.0),
    ///     consider_tooth_tip_leakage: true,
    /// }
    /// .try_into()
    /// .unwrap();
    ///
    /// let air_gap_slotted = SlottedAirGap::new(36, false, CarterFactorModel::Bin12, Box::new(slot));
    ///
    /// let core: RotCore = RotCoreBuilder {
    ///     air_gap_radius: Length::new::<millimeter>(55.0),
    ///     yoke_radius: Length::new::<millimeter>(90.0),
    ///     axial_length: Length::new::<millimeter>(165.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 2,
    ///     skew_angle: 0.0,
    ///     air_gap: Box::new(air_gap_slotted),
    ///     flux_barrier: None,
    /// }
    /// .try_into()
    /// .unwrap();
    ///
    /// let coil_layout = CoilLayout::Quadruple;
    ///
    /// let zones: Vec<PositionedZoneContour> = core.winding_zones(&coil_layout).collect();
    /// assert_eq!(zones.len(), usize::from(coil_layout.layers()) * usize::from(core.slots()));
    /// assert_eq!(zones.len(), 4 * 36);
    /// ```
    fn winding_zones(&self, coil_layout: &CoilLayout) -> WindingZones {
        return self
            .air_gap()
            .winding_zones(self.as_core_ref(), coil_layout);
    }

    fn surface_magnets(&self, magnet_assembly: &MagnetAssembly, split: bool) -> Magnets {
        return self
            .air_gap()
            .surface_magnets(magnet_assembly, self.as_core_ref(), split);
    }

    /// Returns the total number of surface magnets mounted on `self`.
    fn num_surface_magnets(&self, magnet_assembly: &MagnetAssembly) -> usize {
        return usize::from(self.poles()) * magnet_assembly.num_magnets();
    }

    /// Returns the total mass of all surface magnets mounted on `self`.
    fn mass_surface_magnets(&self, magnet_assembly: &MagnetAssembly) -> Mass {
        return self.poles() as f64 * magnet_assembly.mass();
    }

    fn starts_in_d_axis(&self) -> bool {
        self.flux_barrier()
            .map_or(false, |fb| fb.starts_in_d_axis(self.as_core_ref()))
    }

    fn interior_magnets(&self, split: bool) -> Magnets {
        match self.flux_barrier() {
            Some(fb) => fb.interior_magnets(self.as_core_ref(), split),
            None => {
                // Placeholder
                Magnets::Other(Box::new([].into_iter())).into()
            }
        }
    }

    fn interior_magnet_assemblies(&self) -> &[MagnetAssembly] {
        match self.flux_barrier() {
            Some(fb) => fb.magnet_assemblies(self.as_core_ref()),
            None => &[],
        }
    }

    fn mass_interior_magnets(&self) -> Mass {
        let assemblies = self.interior_magnet_assemblies();

        let mut mass = Mass::new::<kilogram>(0.0);
        for mag_idx in self.interior_magnets(false).map(|p| p.magnet_idx) {
            mass += assemblies
                .get(mag_idx)
                .map(|m| m.mass())
                .unwrap_or(Mass::new::<kilogram>(0.0))
        }
        return mass;
    }

    /// Returns the number of slots at the air gap.
    ///
    /// This method forwards to [`AirGap::slots`], using `self` as the second
    /// argument.
    fn slots(&self) -> u16 {
        return self.air_gap().slots(self.as_core_ref());
    }

    /// Calculate the slot opening factor for a non-slotted core according to
    /// eq. (1.2.63) in [MVP08]
    fn slot_opening_factor(&self, slots: u16, ordinal: f64) -> f64 {
        return self
            .air_gap()
            .slot_opening_factor(slots, ordinal, self.as_core_ref());
    }

    /// Returns the current displacement coefficients for a winding mounted on
    /// `self`
    ///
    /// This method forwards to [`AirGap::current_displacement_coefficients`],
    /// using `self` as the second argument. If the
    /// [`air_gap`](CoreExt::air_gap) is a
    /// [`SlottedAirGap`](crate::air_gap::SlottedAirGap), the
    /// [`Slot::current_displacement_coefficients`] method gets invoked. See its
    /// docstring for a general discussion of current displacement.
    fn current_displacement_coefficients(&self) -> CurrentDisplacementCalculator {
        return self
            .air_gap()
            .current_displacement_coefficients(self.as_core_ref());
    }

    /// Returns the slot type of the core, if the core is slotted.
    ///
    /// This method forwards to [`AirGap::slot`], using `self` as the second
    /// argument.
    fn slot(&self) -> Option<&dyn Slot> {
        return self.air_gap().slot(self.as_core_ref());
    }

    /// Returns the tooth height of the core.
    ///
    /// This method forwards to [`AirGap::tooth_height`] with `self` as the
    /// second argument. Usually, the returned value is the [`Slot::height`], as
    /// shown below, but the specific implementation may vary.
    #[doc = ""]
    #[cfg_attr(feature = "doc-images", doc = "![Yoke height][cad_yoke_tooth_height]")]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_yoke_tooth_height",
            "docs/img/cad_yoke_tooth_height.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    fn tooth_height(&self) -> Length {
        return self.air_gap().tooth_height(self.as_core_ref());
    }

    fn tooth_width_at(&self, height: Length) -> Length {
        return self.air_gap().tooth_width_at(self.as_core_ref(), height);
    }

    /// Returns whether a winding can be mounted on the core or not.
    ///
    /// A winding can be mounted if [`CoreExt::slots`] is not zero.
    fn is_windable(&self) -> bool {
        return self.slots() != 0;
    }

    /// Returns whether the core is slotted.
    ///
    /// This method is implemented as `self.slot().is_some()`, i.e. the core is
    /// slotted if [`CoreExt::slot`] doesn't return `None`.
    fn slotted(&self) -> bool {
        return self.slot().is_some();
    }

    fn yoke_mass(&self) -> Mass {
        return self.mass() - self.teeth_mass();
    }

    /// Return the air gap area of the core. In case of a linear core, this
    /// equals the core length times the core width. In case of a rotatory
    /// core, this equals the air gap outline times the core length
    fn air_gap_area(&self) -> Area {
        return self.axial_length() * self.air_gap_width();
    }

    /// Return the air gap length of the axial cross section. This is the core
    /// air gap outline for a rotary core and the core width for a linear
    /// core
    fn air_gap_length(&self) -> Length {
        return self.air_gap_area() / self.axial_length();
    }

    #[cfg(feature = "cairo")]
    fn drawable(&self) -> planar_geo::draw::DrawableCow<'_> {
        let mut style = planar_geo::draw::Style::default();
        style.background_color = crate::GRAY;
        let shape = self.shape();
        return planar_geo::draw::DrawableCow::new(shape, style);
    }

    /// Calculate the electromagnetically active axial length
    fn iron_length(&self) -> Length {
        return self.iron_fill_factor() * self.axial_length();
    }

    /// Calculate the mass of the core
    fn mass(&self) -> Mass {
        return self.cross_section_area()
            * self.iron_length()
            * self.material().mass_density().get(&[]);
    }

    /// Return the cross section area of the core
    fn volume(&self) -> Volume {
        return self.cross_section_area() * self.axial_length();
    }

    /// Return the cross section area of the core
    fn cross_section_area(&self) -> Area {
        return Area::new::<square_meter>(self.shape().area());
    }

    /// Returns the skew factor of the core for the given ordinal.
    ///
    /// This methods forwards to the free function [`skew_factor`] with
    /// [`CoreExt::skew_angle`] and [`CoreExt::num_segments`] as the third and
    /// fourth argument. See the docstring of [`skew_factor`] for details and
    /// examples.
    fn skew_factor(&self, ordinal: usize) -> f64 {
        return skew_factor(ordinal, self.skew_angle(), self.num_segments());
    }

    /**
    Returns the length of a half-coil turn inside the core. If the core is not skewed, this value is equal to the axial length of the core.
    If the core is skewed, the length of the half turn is increased due to the skewing. The corresponding formula is:
    `half_turn_length = axial_length / cos(skew_angle)`.
     */
    fn axial_coil_length(&self) -> Length {
        return self.axial_length() / self.skew_angle().cos();
    }

    /// Approximates the mass of all core teeth
    fn teeth_mass(&self) -> Mass {
        return self.tooth_mass() * self.slots() as f64;
    }

    /// Approximates the mass of a single core tooth
    /// Takes tooth width at middle of slot height -> therefore rough
    /// approximation
    fn tooth_mass(&self) -> Mass {
        match self.slot() {
            Some(slot) => {
                let half_hight =
                    slot.opening_height() + 0.5 * (slot.height() - slot.opening_height());
                return self.material().mass_density().get(&[])
                    * self.iron_length()
                    * self.tooth_height()
                    * self.tooth_width_at(half_hight);
            }
            None => return Mass::new::<kilogram>(0.0),
        }
    }

    /**
    Returns an iterator over the slotting ordinals.

    For details, see the docstring of [`SlottingOrdinals`].
     */
    fn slotting_ordinals(&self) -> SlottingOrdinals {
        return SlottingOrdinals::new(self.slots(), self.pole_pairs());
    }
}

/**
Calculates the skew factor for a mechanical ordinal for an either continuously
skewed or discretized core.

One way to suppress unwanted harmonics of the magnetic air gap field is to
"skew" or "stagger" a magnetic core. The docstring of [`CoreExt::skew_angle`]
contains background information.

This function calculates the "skew factor" for an harmonic with the specified
mechanical `ordinal` where the core is skewed by the `skew_angle`. Multiplying
the skew factor with the amplitude of that harmonic calculated for the unskewed
core returns its resulting (actual) amplitude. The mechanical ordinal is related
to the electrical ordinal via:

```ignore
mechanical_ordinal = electrical_ordinal * pole_pairs
```

i.e. the electrical ordinal gives the number of maxima of the sinusoidal curve
over one pole pair and the mechanical ordinal the number of maxima over the
entire air gap.

If `num_segments` is zero, the core is continuously twisted along its axial
length, otherwise it is composed of `num_segments` straight segments which are
shifted by `skew_angle / num_segments` against each other. In particular, this
means that for `num_segments`, the resulting skew factor is always 1 (as the
core is effectively unskewed). If the number of segments approach infinity, the
core is effectively continuously skewed and the resulting skew factor is
identical to that of `num_segments = 0`. These relations can be directly seen
from the formula for the staggered skew factor taken from
[\[1\]](#skew_factor_1), eq. (3):

```ignore
skew_factor = sin(0.5 * ordinal * skew_angle) / (num_segments * sin(0.5 * ordinal * skew_angle / num_segments))
```

For `num_segments = 0`, the formula simplifies to [\[2\]](#skew_factor_2), eq.
(6.5-18):

```ignore
skew_factor = sin(0.5 * ordinal * skew_angle) / (0.5 * ordinal * skew_angle)
```

# Literature
<a id="skew_factor_1">\[1\]</a>
Huth, Gerhard: Nutrastung von permanenterregten AC-Servomotoren mit gestaffelter
Rotoranordnung, Electrical Engineering 78 (1995), p. 391-397, Springer-Verlag

<a id="skew_factor_2">\[2\]</a>
Binder, Andreas: Elektrische Maschinen und Antriebe (2012), Springer-Verlag,
Berlin Heidelberg

# Examples

## Continuous skewing

A core with 15 slots and 5 pole pairs produces cogging torque harmonics with
the mechanical ordinals 15, 30, 45 and so on due to the slotting. These can be
suppressed by skewing with a full slot pitch (360 / 15 = 24 degree)

```
use std::f64::consts::TAU;
use stem_core::core::skew_factor;
use approx::assert_abs_diff_eq;

let slots = 15;
let pole_pairs = 5;
let angle = TAU / slots as f64;
let num_segments = 0;

// All cogging harmonics are fully suppressed
for k in 1..100 {
    assert_abs_diff_eq!(skew_factor(slots * k, angle, num_segments), 0.0, epsilon = 1e-5);
}

// Other harmonics like the first stator harmonic (which creates the torque)
// are reduced as well.
assert_abs_diff_eq!(skew_factor(pole_pairs, angle, num_segments), 0.82699, epsilon = 1e-5);
```

If especially the 30th ordinal is problematic, it might be more sensible to
skew by 360 / 30 = 12 degree. This reduces the losses for the first harmonic
massively while still fully suppressing the 30th and its multiples.

```
use std::f64::consts::TAU;
use stem_core::core::skew_factor;
use approx::assert_abs_diff_eq;

let slots = 15;
let pole_pairs = 5;
let angle = 0.5 * TAU / slots as f64;
let num_segments = 0;

// Every second cogging harmonic is still fully suppressed
for k in 1..100 {
    assert_abs_diff_eq!(skew_factor(2 * slots * k, angle, num_segments), 0.0, epsilon = 1e-5);
}

// Torque-creating harmonic is much less affected
assert_abs_diff_eq!(skew_factor(pole_pairs, angle, num_segments), 0.95493, epsilon = 1e-5);
```

## Staggering

As discussed above, having only one segment is equal to not skewing at all:

```
use std::f64::consts::TAU;
use stem_core::core::skew_factor;
use approx::assert_abs_diff_eq;

let slots = 15;
let pole_pairs = 5;
let angle = 0.5 * TAU / slots as f64;
let num_segments = 1;

// No suppression of any ordinal
for k in 1..100 {
    assert_abs_diff_eq!(skew_factor(2 * slots * k, angle, num_segments), 1.0, epsilon = 1e-5);
}
assert_abs_diff_eq!(skew_factor(pole_pairs, angle, num_segments), 1.0, epsilon = 1e-5);
```

With two segments, the 30th ordinal can already be suppressed, but some of its
multiples aren't. By increasing the number of segments further, more and more
of these are suppressed as well. For a sufficiently high number of segments,
the staggered rotor behaves like the skewed one and suppresses all multiples.

```
use std::f64::consts::TAU;
use stem_core::core::skew_factor;
use approx::assert_abs_diff_eq;

let slots = 15;
let pole_pairs = 5;
let angle = 0.5 * TAU / slots as f64;

// Two segments
assert_abs_diff_eq!(skew_factor(2 * slots, angle, 2), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(4 * slots, angle, 2), -1.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(6 * slots, angle, 2), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(8 * slots, angle, 2), 1.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(pole_pairs, angle, 2), 0.96592, epsilon = 1e-5);

// Three segments
assert_abs_diff_eq!(skew_factor(2 * slots, angle, 3), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(4 * slots, angle, 3), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(6 * slots, angle, 3), 1.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(8 * slots, angle, 3), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(pole_pairs, angle, 3), 0.95979, epsilon = 1e-5);

// Four segments
assert_abs_diff_eq!(skew_factor(2 * slots, angle, 4), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(4 * slots, angle, 4), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(6 * slots, angle, 4), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(8 * slots, angle, 4), -1.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(pole_pairs, angle, 4), 0.95766, epsilon = 1e-5);

// 100 segments
assert_abs_diff_eq!(skew_factor(2 * slots, angle, 100), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(4 * slots, angle, 100), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(6 * slots, angle, 100), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(8 * slots, angle, 100), 0.0, epsilon = 1e-5);
assert_abs_diff_eq!(skew_factor(pole_pairs, angle, 100), 0.95493, epsilon = 1e-5); // Equals skewed case
```
 */
pub fn skew_factor(ordinal: usize, skew_angle: f64, num_segments: usize) -> f64 {
    if skew_angle == 0.0 {
        return 1.0;
    } else {
        let arg = ordinal as f64 * skew_angle / 2.0;
        if num_segments == 0 {
            return arg.sin() / arg;
        } else {
            return arg.sin() / (num_segments as f64 * (arg / num_segments as f64).sin());
        }
    }
}

/**
An iterator over the slotting ordinals of a core.

When moving along the [`CoreExt::air_gap_width`] of a core, the slot openings
cause a variation in the magnetic resistance / reluctance of the air gap.
Plotting the air gap _permeance_ (inverse reluctance) over the air gap width
will result in a straight line over the tooth heads interrupted by sudden drops
in the slot opening area. This graph can be used to analytically assess the
influence of the tooth head / slot opening geometry on phenomena such as cogging
torque. To do that, the graph is disassembled into its harmonics via Fourier
transformation.

This iterator returns the "electrical" ordinals of those harmonics, i.e. the
ordinal is normalized to a pole pair. To obtain the "mechanical" ordinals,
simply multiply the electrical ordinals by the number of pole pairs.

The ordinals are calculated by [\[1\]](#SlottingOrdinals_1), eq. (8):
`o = k * N / p`
where
`k = 0, 1, 2, ...`
`N`: Number of slots
`p`: Number of pole pairs

Since `k` goes from 0 to infinity, the number of ordinals and therefore this
iterator are also infinite (although it will panic / overflow when
[`usize::MAX`] items have been requested).

The returned iterator items are [`Ratio`](num::rational::Ratio)s instead of
floating point numbers so the underlying physical meaning is clearly visible.
To convert the ratio into a floating point number, use:
`*ratio.denom() as f64 / *ratio.numer() as f64`

# Literature
<a id="SlottingOrdinals_1">\[1\]</a>
Huth, Gerhard: Nutrastung von permanenterregten AC-Servomotoren mit gestaffelter
Rotoranordnung, Electrical Engineering 78 (1995), p. 391-397, Springer-Verlag

# Examples

```
use stem_core::core::SlottingOrdinals;
use num::rational::Ratio;

// Unslotted core
let mut iter = SlottingOrdinals::new(0, 1);
assert_eq!(iter.next(), None);

// 12 slots and 4 pole pairs => Number of slots per pole pair is 3
let mut iter = SlottingOrdinals::new(12, 4);
assert_eq!(iter.next(), Some(Ratio::new(3, 1)));
assert_eq!(iter.next(), Some(Ratio::new(6, 1)));
assert_eq!(iter.next(), Some(Ratio::new(9, 1)));
assert_eq!(iter.next(), Some(Ratio::new(12, 1)));

// 12 slots and 5 pole pairs => Number of slots per pole pair is 12 / 5 = 2.4
let mut iter = SlottingOrdinals::new(12, 5);
assert_eq!(iter.next(), Some(Ratio::new(12, 5)));
assert_eq!(iter.next(), Some(Ratio::new(24, 5)));
assert_eq!(iter.next(), Some(Ratio::new(36, 5)));
assert_eq!(iter.next(), Some(Ratio::new(48, 5)));

// 24 slots and 10 pole pairs => Number of slots per pole pair is 24 / 10 = 2.4
let mut iter = SlottingOrdinals::new(24, 10);
assert_eq!(iter.next(), Some(Ratio::new(12, 5)));
assert_eq!(iter.next(), Some(Ratio::new(24, 5)));
assert_eq!(iter.next(), Some(Ratio::new(36, 5)));
assert_eq!(iter.next(), Some(Ratio::new(48, 5)));
```
 */
#[derive(Debug, Clone, Copy)]
pub struct SlottingOrdinals {
    slots: u16,
    pole_pairs: u16,
    counter: usize,
}

impl SlottingOrdinals {
    /// Creates a new instance of the [`SlottingOrdinals`] iterator.
    pub fn new(slots: u16, pole_pairs: u16) -> Self {
        /*
        Calculate the least common multiple between slots and pole pairs ->
        This is the "base" configuration of the stator after which it simply repeats itself.
         */
        let gcd = num::integer::gcd(slots, pole_pairs);
        let slots = slots / gcd;
        let pole_pairs = pole_pairs / gcd;
        return SlottingOrdinals {
            slots,
            pole_pairs,
            counter: 0,
        };
    }
}

impl Iterator for SlottingOrdinals {
    type Item = num::rational::Ratio<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slots == 0 {
            // Unslotted core
            return None;
        } else {
            let ordinal = self.nth(self.counter);
            self.counter += 1;
            return ordinal;
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        return Some(num::rational::Ratio::new(
            (n + 1) * usize::from(self.slots),
            usize::from(self.pole_pairs),
        ));
    }
}
