/*!
This module provides the [`SlottedAirGap`] for creating a "slotted" air gap with
[`Slot`]s inserted into the air gap surface.
 */
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Slotted linear and rotary core][lin_and_rot_core_slotted.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core_slotted.svg", "docs/img/lin_and_rot_core_slotted.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
This struct implements the [`AirGap`] trait and can therefore be used to build
magnetic cores. See the struct docstring for more.

The [`CarterFactorModel`] enables the usage of different algorithms for
calculating the [`carter_factor`](AirGap::carter_factor) of a core. It is used
as an argument when creating a [`SlottedAirGap`].
*/

use std::f64::consts::FRAC_2_PI;

use crate::planar_geo;
use planar_geo::prelude::*;
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::{
    coil_layout::CoilLayout, current_displacement::CurrentDisplacementCalculator,
    prelude::stem_material::prelude::*, slot::Slot,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    air_gap::AirGap,
    core::{CoreExt, CoreRef, LinCore, RotCore},
    error::Error,
    magnets::{EvenlyDistributedMagnets, Magnets},
    winding_zones::{WindingZones, WindingZonesEqSpaced},
};

/// An enum providing different models for calculating the
/// [`carter_factor`](CoreExt::carter_factor) of a [`SlottedAirGap`].
///
/// The _Carter factor_ `kc` describes the effect of non-smooth (e.g. slotted)
/// air gaps contours on the magnetic resistance / reluctance of the air gap.
/// The magnetically effective air gap width can be calculated as
/// kc_stator_core * kc_rotor_core * geometric_air_gap_width` with both factors
/// being equal to or larger than 1.
///
/// This enum provides multiple different models to calculate `k_c` which are
/// described in the variant docstrings. These models are implemented in the
/// [`CarterFactorModel::eval`] method, which in turn is used inside
/// [`SlottedAirGap::carter_factor`]. The "best" model depends heavily on the
/// particular use case, hence it is recommended to try out different models and
/// see which one delivers the most realistic results.
///
/// Additional models may be added in future releases. Therefore, users should
/// not rely on this enum being exhaustive when matching against it. For
/// calculating the Carter factor, use [CarterFactorModel::eval] rather than
/// reproducing the model-specific calculations.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CarterFactorModel {
    /// Carter factor model from Binder, Andreas: Elektrische Maschinen und
    /// Antriebe (2012), Springer-Verlag, Berlin Heidelberg, section 3.6:
    ///
    /// `k_c = slot_pitch / (slot_pitch - air_gap_width * zeta)` (3.6-1)
    ///
    /// with `zeta = 2 / pi * (h * arctan(h/2) - ln(1+(h/2)²))` (3.6-2)
    /// and `h = slot_opening_width / air_gap_width`.
    Bin12,
    /// Carter factor model from Müller, G., Vogt, K. and Ponick, B.: Berechnung
    /// elektrischer Maschinen, 6th edition, Wiley-VCH, 2008, section 2.3.2.2:
    ///
    /// `k_c = slot_pitch / (slot_pitch - slot_opening_width * gamma)` (2.3.19)
    ///
    /// with `gamma = 1 / (1 + 5 * air_gap_width / slot_opening_width)`
    /// (2.3.20).
    MVP08,
}

impl CarterFactorModel {
    /// Calculates the Carter factor `kc` using one of the
    /// [`CarterFactorModel`]s.
    ///
    /// The `air_gap_width` is the geometric distance between stator and rotor
    /// core. If stator and rotor are rotary cores, this is simply the absolute
    /// difference between their respective
    /// [`air_gap_radii`](crate::core::RotCore::air_gap_radius). The other
    /// arguments describe the air gap geometry; they are described in
    /// detail within the docstrings of the following methods:
    /// - `slot_opening_width`: [`Slot::opening_width`]
    /// - `slot_pitch`: [`CoreExt::slot_pitch`]
    ///
    /// # Examples
    ///
    /// ```
    /// use approx::assert_abs_diff_eq;
    ///
    /// use stem_core::prelude::*;
    ///
    /// // A fairly small slot opening
    /// let air_gap_width = Length::new::<millimeter>(1.0);
    /// let slot_opening_width = Length::new::<millimeter>(2.0);
    /// let slot_pitch = Length::new::<millimeter>(10.0);
    ///
    /// assert_abs_diff_eq!(CarterFactorModel::Bin12.eval(air_gap_width, slot_opening_width, slot_pitch), 1.059179, epsilon = 1e-6);
    /// assert_abs_diff_eq!(CarterFactorModel::MVP08.eval(air_gap_width, slot_opening_width, slot_pitch), 1.060606, epsilon = 1e-6);
    ///
    /// // A large slot opening (open slot)
    /// let air_gap_width = Length::new::<millimeter>(1.0);
    /// let slot_opening_width = Length::new::<millimeter>(5.0);
    /// let slot_pitch = Length::new::<millimeter>(10.0);
    ///
    /// assert_abs_diff_eq!(CarterFactorModel::Bin12.eval(air_gap_width, slot_opening_width, slot_pitch), 1.338269, epsilon = 1e-6);
    /// assert_abs_diff_eq!(CarterFactorModel::MVP08.eval(air_gap_width, slot_opening_width, slot_pitch), 1.333333, epsilon = 1e-6);
    /// ```
    pub fn eval(
        &self,
        air_gap_width: Length,
        slot_opening_width: Length,
        slot_pitch: Length,
    ) -> f64 {
        match self {
            Self::Bin12 => {
                let h = f64::from(slot_opening_width / air_gap_width);
                let zeta = FRAC_2_PI * (h * (0.5 * h).atan() - (1.0 + 0.25 * h.powi(2)).ln());
                return f64::from(slot_pitch / (slot_pitch - zeta * air_gap_width));
            }
            Self::MVP08 => {
                let gamma = slot_opening_width / (slot_opening_width + 5.0 * air_gap_width);
                return f64::from(slot_pitch / (slot_pitch - gamma * slot_opening_width));
            }
        }
    }
}

/**

*/
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Linear and rotary core with a slotted air gap][lin_and_rot_core_slotted]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "lin_and_rot_core_slotted",
        "docs/img/lin_and_rot_core_slotted.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**
TODO
 */
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SlottedAirGap {
    /// Number of slots of the air gap.
    pub slots: u16,
    /// Whether the air gap surface starts in the middle of a slot or inbetween
    /// two slots.
    ///
    /// The image below shows the influence of this parameter on the core shape
    /// for a linear core:
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Effect of the starts_in_slot_middle parameter][lin_slotted_core_slot_vs_tooth_middle]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "lin_slotted_core_slot_vs_tooth_middle",
            "docs/img/lin_slotted_core_slot_vs_tooth_middle.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// If a [`CoilLayout`] cannot be separated horizontally (e.g. in case of a
    /// ([`CoilLayout::Single`])), the layers will protrude outside the air gap
    /// for a linear core. This is obviously not desirable, which is why
    /// this parameter should generally only be `true` for coil layouts
    /// which can be separated horizontally
    /// ([`CoilLayout::DoubleHorizontal`], [`CoilLayout::Quadruple`]).
    /// For a rotary core, this is not the case, as there the parameter only
    /// influences whether the first slot is positioned on the x-axis or not.
    pub starts_in_slot_middle: bool,
    /// The model used in the implementation of [`AirGap::carter_factor`]. See
    /// the docstrings of that method and of [`CarterFactorModel`] itself for
    /// more.
    pub carter_factor_model: CarterFactorModel,
    /// The [`Slot`] of the slotted air gap. Its [`outline`](Slot::outline) is
    /// used to create the core shape.
    pub slot: Box<dyn Slot>,
}

impl SlottedAirGap {
    /// Creates a new [`SlottedAirGap`].
    ///
    /// This is a convenience alternative to using the native struct constructor
    /// directly. See the documentation for the struct fields for details.
    pub fn new(
        slots: u16,
        starts_in_slot_middle: bool,
        carter_factor_model: CarterFactorModel,
        slot: Box<dyn Slot>,
    ) -> Self {
        return Self {
            slots,
            starts_in_slot_middle,
            carter_factor_model,
            slot,
        };
    }

    /// Returns a reference to the
    /// [`SlottedAirGap::slot`](struct.SlottedAirGap.html#structfield.slot)
    /// field.
    ///
    /// This is a convenience method to get a reference to the [`Slot`] trait
    /// object.
    pub fn slot(&self) -> &dyn Slot {
        return &*self.slot;
    }

    /// Returns the core shape for a linear core, if the combination of `self`
    /// and `core` results in a valid shape.
    fn shape_lin(&self, core: &LinCore) -> Result<Shape, Error> {
        // Returns [right half, left half]
        fn split_vertical(ps: Polysegment) -> impl Iterator<Item = Polysegment> {
            let bb = ps.bounding_box();

            let verts_par = [[0.0, bb.ymin() - 1.0], [0.0, bb.ymax() + 1.0]];
            let vertical_line = Polysegment::from_points(verts_par.as_slice());
            let separated =
                ps.intersection_cut(&vertical_line, DEFAULT_EPSILON, DEFAULT_MAX_RELATIVE);

            return separated.into_iter().rev().filter(|ps| !ps.is_empty());
        }

        let is_open = self.slot.is_open();

        let mut slot_iter = WindingZonesEqSpaced::<Polysegment, true>::from_slot(
            core.air_gap_length(),
            self.slots,
            &*self.slot,
            self.starts_in_slot_middle,
            true,
        );

        if is_open {
            let mut ps = Polysegment::new();

            if let Ok(ls) = LineSegment::new(
                [core.width().get::<meter>(), core.height().get::<meter>()],
                [0.0, core.height().get::<meter>()],
            ) {
                ps.push_back(ls.into());
            }

            let mut halfes = if self.starts_in_slot_middle
                && let Some(first_slot) = slot_iter.next()
            {
                Some(split_vertical(first_slot))
            } else {
                None
            };

            if let Some(mut right_half) = halfes.as_mut().map(|i| i.next()).flatten() {
                ps.append(&mut right_half);
            }

            if !self.starts_in_slot_middle {
                ps.extend_back([0.0, 0.0]);
            }

            for mut slot in slot_iter {
                ps.append(&mut slot);
            }

            if !self.starts_in_slot_middle {
                ps.extend_back([core.width().get::<meter>(), 0.0]);
            }

            if let Some(mut left_half) = halfes.as_mut().map(|i| i.next()).flatten() {
                left_half.translate([core.width().get::<meter>(), 0.0]);
                ps.append(&mut left_half);
            }

            return Shape::try_from(ps).map_err(From::from);
        } else {
            let mut shape = if self.starts_in_slot_middle
                && let Some(first_slot) = slot_iter.next()
            {
                let mut halfes = split_vertical(first_slot);

                let mut ps = Polysegment::new();
                if let Ok(ls) = LineSegment::new(
                    [core.width().get::<meter>(), core.height().get::<meter>()],
                    [0.0, core.height().get::<meter>()],
                ) {
                    ps.push_back(ls.into());
                }
                if let Some(mut left_half) = halfes.next() {
                    ps.append(&mut left_half);
                }
                ps.extend_back([0.0, 0.0]);
                ps.extend_back([core.width().get::<meter>(), 0.0]);
                if let Some(mut right_half) = halfes.next() {
                    right_half.translate([core.width().get::<meter>(), 0.0]);
                    ps.append(&mut right_half);
                }
                Shape::try_from(ps)?
            } else {
                let contour = Contour::rectangle(
                    [0.0, 0.0],
                    [core.width().get::<meter>(), core.height().get::<meter>()],
                );
                Shape::try_from(contour)?
            };

            // Add all of the closed slots
            for slot_contour in slot_iter {
                shape.add_hole(slot_contour.into())?;
            }

            return Ok(shape);
        }
    }

    /// Returns the core shape for a rotary core, if the combination of `self`
    /// and `core` results in a valid shape.
    fn shape_rot(&self, core: &RotCore) -> Result<Shape, Error> {
        let yoke = ArcSegment::circle([0.0, 0.0], core.yoke_radius().get::<meter>())?;
        let air_gap_radius = core.air_gap_radius().get::<meter>();
        let is_open = self.slot.is_open();

        let slot_iter = WindingZonesEqSpaced::<Polysegment, false>::from_slot(
            core.air_gap_length(),
            self.slots,
            &*self.slot,
            self.starts_in_slot_middle,
            core.is_outer(),
        );

        if is_open {
            let mut air_gap = Polysegment::new();
            let mut last_stop_opt = None;

            for mut slot_outline in slot_iter {
                if let Some(last_stop) = last_stop_opt {
                    if let Some(next_start) = slot_outline.front().map(|s| s.start()) {
                        if let Ok(a) = ArcSegment::from_start_stop_center_radius(
                            last_stop,
                            next_start,
                            [0.0, 0.0],
                            air_gap_radius,
                            false,
                        ) {
                            air_gap.push_back(a.into());
                        }
                    }
                }

                last_stop_opt = slot_outline.back().map(|s| s.stop());
                air_gap.append(&mut slot_outline);
            }

            // Close the air gap outline. To close the line, we need to connect
            // from stop to start.
            if let Some(start) = air_gap.front().map(|s| s.start()) {
                if let Some(stop) = air_gap.back().map(|s| s.stop()) {
                    if let Ok(a) = ArcSegment::from_start_stop_center_radius(
                        stop,
                        start,
                        [0.0, 0.0],
                        air_gap_radius,
                        false,
                    ) {
                        air_gap.push_back(a.into());
                    }
                }
            } else {
                // No segment in the air gap -> air gap is a simple circle
                air_gap.push_back(ArcSegment::circle([0.0, 0.0], air_gap_radius)?.into());
            }

            if core.is_outer() {
                return Shape::new(vec![yoke.into(), air_gap.into()]).map_err(From::from);
            } else {
                return Shape::new(vec![air_gap.into(), yoke.into()]).map_err(From::from);
            }
        } else {
            let air_gap = ArcSegment::circle([0.0, 0.0], air_gap_radius)?;
            let mut shape = if core.is_outer() {
                Shape::new(vec![yoke.into(), air_gap.into()])?
            } else {
                Shape::new(vec![air_gap.into(), yoke.into()])?
            };

            // Add all of the closed slots
            for slot_contour in slot_iter {
                shape.add_hole(slot_contour.into())?;
            }
            return Ok(shape);
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl AirGap for SlottedAirGap {
    fn num_segments(&self, _: CoreRef<'_>) -> usize {
        return 0;
    }

    fn winding_zones(&self, core: CoreRef<'_>, coil_layout: &CoilLayout) -> WindingZones {
        match core {
            CoreRef::Lin(_) => WindingZones::WindingZonesEqSpacedLin(WindingZonesEqSpaced::<
                Contour,
                true,
            >::from_slot(
                core.air_gap_length(),
                core.slots(),
                &*self.slot,
                coil_layout,
                self.starts_in_slot_middle,
                true,
            )),
            CoreRef::Rot(rot) => WindingZones::WindingZonesEqSpacedRot(WindingZonesEqSpaced::<
                Contour,
                false,
            >::from_slot(
                core.air_gap_length(),
                core.slots(),
                &*self.slot,
                coil_layout,
                self.starts_in_slot_middle,
                rot.is_outer(),
            )),
        }
    }

    fn slots(&self, _: CoreRef<'_>) -> u16 {
        return self.slots;
    }

    fn slot(&self, _core: CoreRef<'_>) -> Option<&dyn Slot> {
        return Some(&*self.slot);
    }

    fn surface_magnets(
        &self,
        magnet_assembly: &MagnetAssembly,
        core: CoreRef<'_>,
        split: bool,
    ) -> Magnets {
        match core {
            CoreRef::Lin(_) => EvenlyDistributedMagnets::<true>::from_magnet_assembly(
                core.poles().into(),
                core.air_gap_length(),
                magnet_assembly,
                split,
                true, // Value is ignored anyway if LIN = true
            )
            .into(),
            CoreRef::Rot(core_rot) => EvenlyDistributedMagnets::<false>::from_magnet_assembly(
                core.poles().into(),
                core.air_gap_length(),
                magnet_assembly,
                split,
                core_rot.is_outer(),
            )
            .into(),
        }
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Shape, Error> {
        match core {
            CoreRef::Lin(lin_core) => self.shape_lin(lin_core),
            CoreRef::Rot(rot_core) => self.shape_rot(rot_core),
        }
    }

    fn current_displacement_coefficients(
        &self,
        _core: CoreRef<'_>,
    ) -> CurrentDisplacementCalculator {
        return self.slot.current_displacement_coefficients(50);
    }

    fn carter_factor(&self, core: CoreRef<'_>, air_gap_length: Length) -> f64 {
        return self.carter_factor_model.eval(
            air_gap_length,
            self.slot().opening_width(),
            core.slot_pitch(),
        );
    }

    fn slot_opening_factor(&self, core: CoreRef<'_>, mech_ordinal: i32) -> f64 {
        let slot_pitch = core.slot_pitch();
        return super::slot_opening_factor(
            slot_pitch,
            self.slot.opening_width(),
            self.slots,
            mech_ordinal,
        );
    }
}
