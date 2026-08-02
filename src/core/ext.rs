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
When a [`collision_check`](CoreExt::collision_check) fails, this enum describes
one of the components which collided with another component. See
[`AssemblingFailure`] for more information.
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
An error type created by [`CoreExt::collision_check`] which describes a
collision between two components of an active part (magnetic core,
winding zones, surface magnets, interior magnets). It holds the two colliding
components and the reason for the collision.
 */
#[derive(Clone, Debug)]
pub struct AssemblingFailure {
    /// One of the colliding components.
    pub left_component: Component,
    /// The component which collided with the `left_component`.
    pub right_component: Component,
    /// Reason for the collision.
    pub reason: AssemblingFailureReason,
}

/**
An enum which describes the reason for a collision between the two components of
an active part. It is created as part of the [`AssemblingFailure`] error when a
[`collision_check`](CoreExt::collision_check) fails.
 */
#[derive(Clone, Debug)]
pub enum AssemblingFailureReason {
    /// The two components are overlapping.
    Overlap(Overlap),
    /// One component which should be contained by another one isn't. An example
    /// would be an interior magnet which is not contained by the
    /// [`CoreExt::shape`].
    NotContained(NotContained),
}

impl From<Overlap> for AssemblingFailureReason {
    fn from(value: Overlap) -> Self {
        Self::Overlap(value)
    }
}

impl From<NotContained> for AssemblingFailureReason {
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
    fn iron_fill_factor(&self) -> f64;

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
    fn yoke_height(&self) -> Length;

    /// Return the skew angle of the component. The skewing can be implemented
    /// either within the core itself or in the magnet arrangement.
    fn skew_angle(&self) -> f64;

    /**
    Return the air gap "width" of the core.  In case of a linear core,
    this is the actual width of the stator cross section. For a rotary core,
    this is the air gap outline (radius times 2 pi).
     */
    fn air_gap_width(&self) -> Length;

    /// Returns if the core is linear or rotary.
    fn lin_or_rot(&self) -> LinOrRot;

    /**
    Returns the total axial coil overhang on boths sides of the magnetic core.

    The ASCII art below visualizes the axial coil overhang with equal signs (=).
    If = equals one mm, the return value of `axial_coil_overhang` would be 3 mm.
    ```ignore
         ┌──────┐
    ┌──==│      │=──┐
    │    │ Core │   │ <-- Coil
    └──==│      │=──┘
         └──────┘
    ```

    Axial overhang can be caused by e.g. the end winding insulation. While this
    overhang is part of the end winding, it is not included in the calculation
    of the end winding length of the `IsWinding` trait method `end_winding_half_turn_length`.
    Therefore, in the calculation of the end winding inductance, the coil length is calculated as `axial_coil_overhang` + `end_winding_half_turn_length`.
    This length is not considered in the main inductance calculation.
     */
    fn axial_coil_overhang(&self) -> Length;

    /// Returns a reference to the cross-section shape of `self`.
    fn shape(&self) -> &Shape;

    /// Relative coverage of air gap
    /// If surface_magnet_assembly given -> coverage of surface magnets
    /// If not given -> Coverage of flux barrier or simply half the air gap
    fn pole_coverage(&self, surface_magnet_assembly: Option<&MagnetAssembly>) -> f64;

    // =========================================================================

    /// Returns the total number of poles.
    ///
    /// This is 2 * [`Self::pole_pairs`].
    fn poles(&self) -> u16 {
        return 2 * self.pole_pairs();
    }

    fn collision_check(
        &self,
        coil_layout: &CoilLayout,
        surface_magnet_assembly: Option<&MagnetAssembly>,
        epsilon: f64,
        max_relative: f64,
    ) -> Result<(), AssemblingFailure> {
        let core_shape = self.shape();
        let zones: Vec<PositionedZoneContour> = self.winding_zones(coil_layout).collect();
        if let Some(o) = zones.par_iter().enumerate().find_map_any(|(i1, z1)| {
            if let Ok(overlap) =
                core_shape.contains_any_composite(&z1.contour, epsilon, max_relative)
            {
                return Some(AssemblingFailure {
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
                    return Some(AssemblingFailure {
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
                    return Some(AssemblingFailure {
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
                        return Some(AssemblingFailure {
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
                        return Some(AssemblingFailure {
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
                if let Err(e) = core_shape.contains_shape(&m.shape, epsilon, max_relative) {
                    return Some(AssemblingFailure {
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
                    return Some(AssemblingFailure {
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
    Method forwards to [`AirGap::carter_factor`] converting self into [`CoreRef`] -> See its docstring.
     */
    fn carter_factor(&self, air_gap_width: Length) -> f64 {
        self.air_gap()
            .carter_factor(self.as_core_ref(), air_gap_width)
    }

    /// Returns the number of segments. If this value is 0, the component is
    /// continuously skewed.
    fn num_segments(&self) -> usize {
        return self.air_gap().num_segments(self.as_core_ref());
    }

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
    fn windable(&self) -> bool {
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
    This function returns the `SlottingOrdinals` ordinals iterator for the given core.
    For more details, see the documentation of `SlottingOrdinals`.
     */
    fn slotting_ordinals(&self) -> SlottingOrdinals {
        return SlottingOrdinals::new(self.slots(), self.pole_pairs());
    }
}

/**
Calculate the skew factor for a given ordinal for either a continuously skewed (`num_segments` = 0) or discretly staggered (`num_segments` > 0) electrical machine component.
In case of discrete staggering, each segment is twisted by `skew_angle / num_segments` against its neighboring segments. For an infinite number of segments,
this results in the same skew factor as a continuous skewing. The formulae are taken from [Hut95]. For additional explanation, see also [Bin12], p. 629.

The skew factor represents the normalized superposition of multiple (in case of discrete staggering) or infinitely many (in case of continuous skew) sinusoidals
which are shifted against each other by the skew angle. Therefore, it describes the influence of the skew angle on any time- or space dependent property. Two examples are:
- The flux linkage between stator and rotor caused by currents / permanent magnets
- The cogging torque / torque ripple caused by the stator slotting / the non-sinusoidal field excitation curve.

The "mechanical" ordinal gives the number of repetitions of the sinusoidal in the entire airgap. Dividing by the number of pole pairs gives the "electrical" ordinal.
In this formula, the mechanical ordinal is used.

# Arguments
- ordinal: Mechanical ordinal of the harmonic
- skew_angle: Skew angle in rad
- num_segments: Inner of component segments. If set to 0, the component is continuously skewed.

The calculation formula for a continuously skewed component is taken from [Hut18].

```
use std::f64::consts::PI;
use magnetic_core::skew_factor;
use approx::assert_abs_diff_eq;

// Continuous case:
// This example describes the 15/10 double-layer winding from [Mat22], which has a skew angle of 12 degree (half a slot)
// to suppress the the first cogging torque harmonic (ordinal 30)
let angle = 12.0 / 180.0 * PI;
assert_abs_diff_eq!(skew_factor(5, angle, 0), 0.95493, epsilon = 0.0001); // First stator field harmonic
assert_abs_diff_eq!(skew_factor(25, angle, 0), 0.190985, epsilon = 0.0001); // Fifth stator field harmonic
assert_abs_diff_eq!(skew_factor(35, angle, 0), -0.1364, epsilon = 0.0001); // Seventh stator field harmonic

assert_abs_diff_eq!(skew_factor(30, angle, 0), 0.0, epsilon = 0.0001); // First cogging torque harmonic
assert_abs_diff_eq!(skew_factor(60, angle, 0), 0.0, epsilon = 0.0001); // Second cogging torque harmonic
assert_abs_diff_eq!(skew_factor(90, angle, 0), 0.0, epsilon = 0.0001); // Third cogging torque harmonic

// Staggered case
assert_abs_diff_eq!(skew_factor(30, angle, 1), 1.0, epsilon = 0.0001); // One segment => No suppression

// Two segments are sufficient to supress the first cogging torque harmonic. Using more segments does not change that.
assert_abs_diff_eq!(skew_factor(30, angle, 2), 0.0, epsilon = 0.0001);
assert_abs_diff_eq!(skew_factor(30, angle, 3), 0.0, epsilon = 0.0001);
assert_abs_diff_eq!(skew_factor(30, angle, 4), 0.0, epsilon = 0.0001);

// However, the second cogging torque harmonic requires more segments.
assert_abs_diff_eq!(skew_factor(60, angle, 2), -1.0, epsilon = 0.0001); // Two segments
assert_abs_diff_eq!(skew_factor(60, angle, 3), 0.0, epsilon = 0.0001);
assert_abs_diff_eq!(skew_factor(60, angle, 4), 0.0, epsilon = 0.0001);

// For a large number of segments, the continuous and the staggered case converge
assert_abs_diff_eq!(skew_factor(5, 12.0 / 180.0 * PI, 30), skew_factor(5, 12.0 / 180.0 * PI, 0), epsilon = 0.0001);
```
 */
pub fn skew_factor(ordinal: usize, skew_angle: f64, num_segments: usize) -> f64 {
    if skew_angle == 0.0 {
        return 1.0;
    } else {
        if num_segments == 0 {
            let arg = ordinal as f64 * skew_angle / 2.0;
            return arg.sin() / arg;
        } else {
            let arg = ordinal as f64 * skew_angle / 2.0;
            return arg.sin() / (num_segments as f64 * (arg / num_segments as f64).sin());
        }
    }
}

/**
The slotting of a ferromagnetic core causes permeance variations when moving along the air gap.
This iterator returns the ordinals of the Fourier transformation of the permeance course
(normalized to a polepair, therefore "electrical" ordinals).
The ordinals are calculated by the formula:
`o = k * N / p`
where
`k = 0, 1, 2, ...`
`N`: Inner of slots
`p`: Inner of pole pairs

Since `k` goes from 0 to infinity, the number of ordinals is also infinite. If the ferromagnetic core
is not slotted, the constructor input value of `slots` should be zero, in which case this iterator
returns `None`.

The calculation formula is taken from [Hut95], eq. (8).

# Example:
```
use magnetic_core::SlottingOrdinals;
use num::rational::Ratio;

// Unslotted core
let mut iter = SlottingOrdinals::new(0, 1);
assert_eq!(iter.next(), None);

// 12 slots and 4 pole pairs => Inner of slots per pole pair is 3
let mut iter = SlottingOrdinals::new(12, 4);
assert_eq!(iter.next(), Some(Ratio::new(3, 1)));
assert_eq!(iter.next(), Some(Ratio::new(6, 1)));
assert_eq!(iter.next(), Some(Ratio::new(9, 1)));
assert_eq!(iter.next(), Some(Ratio::new(12, 1)));

// 12 slots and 5 pole pairs => Inner of slots per pole pair is 12 / 5 = 2.4
let mut iter = SlottingOrdinals::new(12, 5);
assert_eq!(iter.next(), Some(Ratio::new(12, 5)));
assert_eq!(iter.next(), Some(Ratio::new(24, 5)));
assert_eq!(iter.next(), Some(Ratio::new(36, 5)));
assert_eq!(iter.next(), Some(Ratio::new(48, 5)));

// 24 slots and 10 pole pairs => Inner of slots per pole pair is 24 / 10 = 2.4
let mut iter = SlottingOrdinals::new(24, 10);
assert_eq!(iter.next(), Some(Ratio::new(12, 5)));
assert_eq!(iter.next(), Some(Ratio::new(24, 5)));
assert_eq!(iter.next(), Some(Ratio::new(36, 5)));
assert_eq!(iter.next(), Some(Ratio::new(48, 5)));
```
 */
pub struct SlottingOrdinals {
    slots: u16,
    pole_pairs: u16,
    counter: usize,
}

impl SlottingOrdinals {
    /**
    Create a new `SlottingOrdinals` iterator for the specified number of slots and pole pairs.
     */
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
