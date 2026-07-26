use compare_variables::compare_variables;
use planar_geo::{prelude::BoundingBox, shape::Shape};
use std::{f64::consts::TAU, sync::Arc};
use stem_magnet::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_mosaic::{deserialize_arc_link, serialize_arc_link};

use super::CoreExt;
use crate::{LinOrRot, air_gap::AirGap, error::IncompatibleFluxBarrier, flux_barrier::FluxBarrier};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RotCoreBuilder"))]
pub struct RotCore {
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    air_gap_radius: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    yoke_radius: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_length: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_coil_overhang: Length,
    iron_fill_factor: f64,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_arc_link",))]
    material: Arc<Material>, // core material
    pole_pairs: u16,
    skew_angle: f64,
    air_gap: Box<dyn AirGap>,
    flux_barrier: Option<Box<dyn FluxBarrier>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    shape: Shape,
}

impl RotCore {
    pub fn new(builder: RotCoreBuilder) -> Result<Self, crate::error::Error> {
        builder.try_into()
    }

    /// Return the air gap radius
    pub fn air_gap_radius(&self) -> Length {
        return self.air_gap_radius;
    }

    /// Return the yoke radius
    pub fn yoke_radius(&self) -> Length {
        return self.yoke_radius;
    }

    pub fn is_outer(&self) -> bool {
        return self.air_gap_radius() < self.yoke_radius();
    }

    /**
    Return the radius of the yoke middle. For a non-slotted core, this is the mean radius of yoke and air gap radius.
    For a slotted core, this is the mean radius between the circle enclosing the tooth feet in the yoke and the yoke radius.
     */
    pub fn yoke_middle_radius(&self) -> Length {
        let outer = self.is_outer();
        let sign = outer as i32 as f64 - (!outer) as i32 as f64;
        return 0.5
            * (self.air_gap_radius()
                + sign * self.air_gap().tooth_height(self.into())
                + self.yoke_radius());
    }

    /// Returns the offset between the coordinate system origin of the core and
    /// that of its slots, if it has any.
    ///
    /// This method returns the `offset` between the coordinate system of a
    /// [`Slot`] and that of `self`:
    ///
    /// `r_core = y_slot + offset`.
    ///
    /// If the slot is closed, `offset` is simply the air gap radius. If the
    /// slot is open, the slot coordinate system is shifted inwards by `delta`
    /// as shown in the image below. This is done because the actual slot
    /// opening height would be larger than [`Slot::opening_height`] because a
    /// rotary core "bends away" from the slot baseline, necessitating the
    /// shift by `delta` to compensate. For an open slot, `offset` is therefore:
    ///
    /// `offset = sqrt(RotCore::air_gap_radius² - (Slot::opening_width/2)²)`
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Offset between the core and the slot coordinate system origin][cad_rot_core_slot_cs_offset]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_rot_core_slot_cs_offset",
            "docs/img/cad_rot_core_slot_cs_offset.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// # Examples
    ///
    /// ```
    /// TODO
    /// ```
    pub fn origin_offset_core_to_slot(&self) -> Length {
        use uom::typenum::P2;
        if let Some(slot) = self.slot() {
            let r = self.air_gap_radius();
            let s = slot.opening_width();
            return (r.powi(P2::new()) - (0.5 * s).powi(P2::new())).sqrt();
        } else {
            return Length::new::<meter>(0.0);
        }
    }

    pub fn set_flux_barrier(
        &mut self,
        flux_barrier: Option<Box<dyn crate::prelude::FluxBarrier>>,
    ) -> Result<(), IncompatibleFluxBarrier> {
        let mut air_gap: Box<dyn AirGap> = Box::new(crate::air_gap::PlainAirGap::default());
        std::mem::swap(&mut air_gap, &mut self.air_gap);
        let mut shape = air_gap.combine(self.as_core_ref()).expect(
            "air gap - core combination produced a valid shape during construction of self",
        );
        std::mem::swap(&mut air_gap, &mut self.air_gap);

        if let Some(mut fb) = flux_barrier {
            let contours = match fb.combine(self.as_core_ref()) {
                Ok(c) => c,
                Err(cause) => {
                    return Err(IncompatibleFluxBarrier {
                        flux_barrier: fb,
                        cause,
                    });
                }
            };

            for hole in contours {
                if let Err(cause) = shape.add_hole(hole) {
                    return Err(IncompatibleFluxBarrier {
                        flux_barrier: fb,
                        cause: cause.into(),
                    });
                }
            }
            self.flux_barrier = Some(fb);
        } else {
            self.flux_barrier = None;
        }

        self.shape = shape;
        return Ok(());
    }
}

impl super::ext::private::Sealed for RotCore {}

impl CoreExt for RotCore {
    fn air_gap(&self) -> &dyn AirGap {
        return &*self.air_gap;
    }

    fn flux_barrier(&self) -> Option<&dyn FluxBarrier> {
        return self.flux_barrier.as_ref().map(|v| &**v);
    }

    fn air_gap_width(&self) -> Length {
        return self.air_gap_radius() * TAU;
    }

    fn axial_length(&self) -> Length {
        return self.axial_length;
    }

    fn yoke_height(&self) -> Length {
        return (self.air_gap_radius() - self.yoke_radius()).abs() - self.tooth_height();
    }

    fn iron_fill_factor(&self) -> f64 {
        return self.iron_fill_factor;
    }

    fn material(&self) -> &Arc<Material> {
        return &self.material;
    }

    fn pole_pairs(&self) -> u16 {
        return self.pole_pairs;
    }

    fn number_segments(&self) -> usize {
        return self.air_gap.number_segments(self.into());
    }

    fn skew_angle(&self) -> f64 {
        return self.skew_angle;
    }

    fn lin_or_rot(&self) -> LinOrRot {
        return LinOrRot::Rot;
    }

    fn axial_coil_overhang(&self) -> Length {
        return self.axial_coil_overhang;
    }

    fn shape<'a>(&'a self) -> &'a Shape {
        return &self.shape;
    }

    fn as_core_ref(&self) -> super::CoreRef<'_> {
        return self.into();
    }

    fn pole_coverage(&self, surface_magnet_assembly: Option<&MagnetAssembly>) -> f64 {
        if let Some(assembly) = surface_magnet_assembly {
            let single_magnet_coverage = crate::magnets::pole_coverage_angle(
                std::iter::once(&*assembly.magnet().shape()),
                self.air_gap_radius.get::<meter>(),
                Length::new::<meter>(0.0),
            );
            return 2.0
                * self.pole_pairs() as f64
                * assembly.num_tangential() as f64
                * single_magnet_coverage
                / TAU;
        } else {
            if let Some(flux_barrier) = &self.flux_barrier {
                return flux_barrier.pole_coverage(self.as_core_ref());
            } else {
                return 0.5;
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct RotCoreBuilder {
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub air_gap_radius: Length, // Air gap radius (in the d-axis)
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub yoke_radius: Length, // Yoke radius
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub axial_length: Length, // Axial length of the core
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub axial_coil_overhang: Length,
    pub iron_fill_factor: f64,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_arc_link"))]
    pub material: Arc<Material>, // core material
    pub pole_pairs: u16,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_angle"))]
    pub skew_angle: f64,
    pub air_gap: Box<dyn AirGap>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub flux_barrier: Option<Box<dyn FluxBarrier>>,
}

impl TryFrom<RotCoreBuilder> for RotCore {
    type Error = crate::error::Error;

    fn try_from(builder: RotCoreBuilder) -> Result<Self, Self::Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero <= builder.axial_coil_overhang)?;
        compare_variables!(0.0 <= builder.iron_fill_factor <= 1.0)?;

        let mut this = RotCore {
            air_gap_radius: builder.air_gap_radius,
            yoke_radius: builder.yoke_radius,
            axial_length: builder.axial_length,
            iron_fill_factor: builder.iron_fill_factor,
            material: builder.material,
            pole_pairs: builder.pole_pairs,
            skew_angle: builder.skew_angle,
            axial_coil_overhang: builder.axial_coil_overhang,
            // Placeholder
            air_gap: builder.air_gap.clone(),
            // Placeholder
            flux_barrier: None,
            // Placeholder
            shape: Shape::from_outer(BoundingBox::new(0.0, 1.0, 0.0, 1.0).into())?,
        };

        let mut ag = builder.air_gap;
        this.shape = AirGap::combine(&mut *ag, this.as_core_ref())?;
        this.air_gap = ag;

        // Check if core and flux barrier are compatible
        if let Some(mut fb) = builder.flux_barrier {
            let contours = FluxBarrier::combine(&mut *fb, this.as_core_ref())?;
            for contour in contours {
                this.shape.add_hole(contour)?;
            }
            this.flux_barrier = Some(fb);
        }

        return Ok(this);
    }
}
