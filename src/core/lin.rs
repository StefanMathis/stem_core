use compare_variables::compare_variables;
use planar_geo::{prelude::BoundingBox, shape::Shape};
use std::sync::Arc;
use stem_magnet::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_mosaic::{deserialize_arc_link, serialize_arc_link};

use super::CoreExt;
use crate::{
    LinOrRot,
    air_gap::{AirGap, PlainAirGap},
    error::IncompatibleFluxBarrier,
    flux_barrier::FluxBarrier,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "LinCoreBuilder"))]
pub struct LinCore {
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    height: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    width: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_length: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_coil_overhang: Length,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_angle"))]
    skew_angle: f64,
    iron_fill_factor: f64,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_arc_link",))]
    material: Arc<Material>, // core material
    pole_pairs: u16,
    air_gap: Box<dyn AirGap>,
    flux_barrier: Option<Box<dyn FluxBarrier>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    shape: Shape,
}

impl LinCore {
    pub fn new(builder: LinCoreBuilder) -> Result<Self, crate::error::Error> {
        builder.try_into()
    }

    pub fn height(&self) -> Length {
        return self.height;
    }

    pub fn width(&self) -> Length {
        return self.width;
    }

    pub fn set_flux_barrier(
        &mut self,
        flux_barrier: Option<Box<dyn crate::prelude::FluxBarrier>>,
    ) -> Result<(), IncompatibleFluxBarrier> {
        let mut air_gap: Box<dyn AirGap> = Box::new(PlainAirGap::default());
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

impl super::ext::private::Sealed for LinCore {}

impl CoreExt for LinCore {
    fn air_gap(&self) -> &dyn AirGap {
        return &*self.air_gap;
    }

    fn flux_barrier(&self) -> Option<&dyn FluxBarrier> {
        return self.flux_barrier.as_ref().map(|v| &**v);
    }
    fn axial_length(&self) -> Length {
        return self.axial_length;
    }

    fn air_gap_width(&self) -> Length {
        return self.width;
    }

    fn iron_fill_factor(&self) -> f64 {
        return self.iron_fill_factor;
    }

    fn pole_pairs(&self) -> u16 {
        return self.pole_pairs;
    }

    fn material(&self) -> &Arc<Material> {
        return &self.material;
    }

    fn yoke_height(&self) -> Length {
        return self.height - self.air_gap.tooth_height(self.into());
    }

    fn skew_angle(&self) -> f64 {
        return self.skew_angle;
    }

    fn lin_or_rot(&self) -> LinOrRot {
        return LinOrRot::Lin;
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
            let single_magnet_coverage = BoundingBox::from(&*assembly.magnet().shape()).width();
            return 2.0
                * self.pole_pairs() as f64
                * assembly.num_tangential() as f64
                * single_magnet_coverage
                / self.width().get::<meter>();
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
pub struct LinCoreBuilder {
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub height: Length, // Core height ("y-axis")
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub width: Length, // Core width ("x-axis")
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub axial_length: Length, // Axial length of the core
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub axial_coil_overhang: Length,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_angle"))]
    pub skew_angle: f64,
    pub iron_fill_factor: f64,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_arc_link"))]
    pub material: Arc<Material>, // core material
    pub pole_pairs: u16,
    pub air_gap: Box<dyn AirGap>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub flux_barrier: Option<Box<dyn FluxBarrier>>,
}

impl TryFrom<LinCoreBuilder> for LinCore {
    type Error = crate::error::Error;

    fn try_from(builder: LinCoreBuilder) -> Result<Self, Self::Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero <= builder.axial_coil_overhang)?;
        compare_variables!(0.0 <= builder.iron_fill_factor <= 1.0)?;

        let mut this = LinCore {
            height: builder.height,
            width: builder.width,
            axial_length: builder.axial_length,
            iron_fill_factor: builder.iron_fill_factor,
            material: builder.material,
            pole_pairs: builder.pole_pairs,
            skew_angle: builder.skew_angle,
            axial_coil_overhang: builder.axial_coil_overhang,
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
