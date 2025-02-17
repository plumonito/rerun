// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/rust/api.rs
// Based on "crates/store/re_types/definitions/rerun/blueprint/archetypes/near_clip_plane.fbs".

#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::cloned_instead_of_copied)]
#![allow(clippy::map_flatten)]
#![allow(clippy::needless_question_mark)]
#![allow(clippy::new_without_default)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use ::re_types_core::external::arrow;
use ::re_types_core::SerializationResult;
use ::re_types_core::{ComponentBatch, ComponentBatchCowWithDescriptor};
use ::re_types_core::{ComponentDescriptor, ComponentName};
use ::re_types_core::{DeserializationError, DeserializationResult};

/// **Archetype**: Controls the distance to the near clip plane in 3D scene units.
#[derive(Clone, Debug, Copy)]
pub struct NearClipPlane {
    /// Controls the distance to the near clip plane in 3D scene units.
    ///
    /// Content closer than this distance will not be visible.
    pub near_clip_plane: crate::blueprint::components::NearClipPlane,
}

static REQUIRED_COMPONENTS: once_cell::sync::Lazy<[ComponentDescriptor; 0usize]> =
    once_cell::sync::Lazy::new(|| []);

static RECOMMENDED_COMPONENTS: once_cell::sync::Lazy<[ComponentDescriptor; 1usize]> =
    once_cell::sync::Lazy::new(|| {
        [ComponentDescriptor {
            archetype_name: Some("rerun.blueprint.archetypes.NearClipPlane".into()),
            component_name: "rerun.blueprint.components.NearClipPlaneIndicator".into(),
            archetype_field_name: None,
        }]
    });

static OPTIONAL_COMPONENTS: once_cell::sync::Lazy<[ComponentDescriptor; 1usize]> =
    once_cell::sync::Lazy::new(|| {
        [ComponentDescriptor {
            archetype_name: Some("rerun.blueprint.archetypes.NearClipPlane".into()),
            component_name: "rerun.blueprint.components.NearClipPlane".into(),
            archetype_field_name: Some("near_clip_plane".into()),
        }]
    });

static ALL_COMPONENTS: once_cell::sync::Lazy<[ComponentDescriptor; 2usize]> =
    once_cell::sync::Lazy::new(|| {
        [
            ComponentDescriptor {
                archetype_name: Some("rerun.blueprint.archetypes.NearClipPlane".into()),
                component_name: "rerun.blueprint.components.NearClipPlaneIndicator".into(),
                archetype_field_name: None,
            },
            ComponentDescriptor {
                archetype_name: Some("rerun.blueprint.archetypes.NearClipPlane".into()),
                component_name: "rerun.blueprint.components.NearClipPlane".into(),
                archetype_field_name: Some("near_clip_plane".into()),
            },
        ]
    });

impl NearClipPlane {
    /// The total number of components in the archetype: 0 required, 1 recommended, 1 optional
    pub const NUM_COMPONENTS: usize = 2usize;
}

/// Indicator component for the [`NearClipPlane`] [`::re_types_core::Archetype`]
pub type NearClipPlaneIndicator = ::re_types_core::GenericIndicatorComponent<NearClipPlane>;

impl ::re_types_core::Archetype for NearClipPlane {
    type Indicator = NearClipPlaneIndicator;

    #[inline]
    fn name() -> ::re_types_core::ArchetypeName {
        "rerun.blueprint.archetypes.NearClipPlane".into()
    }

    #[inline]
    fn display_name() -> &'static str {
        "Near clip plane"
    }

    #[inline]
    fn indicator() -> ComponentBatchCowWithDescriptor<'static> {
        static INDICATOR: NearClipPlaneIndicator = NearClipPlaneIndicator::DEFAULT;
        ComponentBatchCowWithDescriptor::new(&INDICATOR as &dyn ::re_types_core::ComponentBatch)
    }

    #[inline]
    fn required_components() -> ::std::borrow::Cow<'static, [ComponentDescriptor]> {
        REQUIRED_COMPONENTS.as_slice().into()
    }

    #[inline]
    fn recommended_components() -> ::std::borrow::Cow<'static, [ComponentDescriptor]> {
        RECOMMENDED_COMPONENTS.as_slice().into()
    }

    #[inline]
    fn optional_components() -> ::std::borrow::Cow<'static, [ComponentDescriptor]> {
        OPTIONAL_COMPONENTS.as_slice().into()
    }

    #[inline]
    fn all_components() -> ::std::borrow::Cow<'static, [ComponentDescriptor]> {
        ALL_COMPONENTS.as_slice().into()
    }

    #[inline]
    fn from_arrow_components(
        arrow_data: impl IntoIterator<Item = (ComponentName, arrow::array::ArrayRef)>,
    ) -> DeserializationResult<Self> {
        re_tracing::profile_function!();
        use ::re_types_core::{Loggable as _, ResultExt as _};
        let arrays_by_name: ::std::collections::HashMap<_, _> = arrow_data
            .into_iter()
            .map(|(name, array)| (name.full_name(), array))
            .collect();
        let near_clip_plane = {
            let array = arrays_by_name
                .get("rerun.blueprint.components.NearClipPlane")
                .ok_or_else(DeserializationError::missing_data)
                .with_context("rerun.blueprint.archetypes.NearClipPlane#near_clip_plane")?;
            <crate::blueprint::components::NearClipPlane>::from_arrow_opt(&**array)
                .with_context("rerun.blueprint.archetypes.NearClipPlane#near_clip_plane")?
                .into_iter()
                .next()
                .flatten()
                .ok_or_else(DeserializationError::missing_data)
                .with_context("rerun.blueprint.archetypes.NearClipPlane#near_clip_plane")?
        };
        Ok(Self { near_clip_plane })
    }
}

impl ::re_types_core::AsComponents for NearClipPlane {
    fn as_component_batches(&self) -> Vec<ComponentBatchCowWithDescriptor<'_>> {
        re_tracing::profile_function!();
        use ::re_types_core::Archetype as _;
        [
            Some(Self::indicator()),
            (Some(&self.near_clip_plane as &dyn ComponentBatch)).map(|batch| {
                ::re_types_core::ComponentBatchCowWithDescriptor {
                    batch: batch.into(),
                    descriptor_override: Some(ComponentDescriptor {
                        archetype_name: Some("rerun.blueprint.archetypes.NearClipPlane".into()),
                        archetype_field_name: Some(("near_clip_plane").into()),
                        component_name: ("rerun.blueprint.components.NearClipPlane").into(),
                    }),
                }
            }),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl ::re_types_core::ArchetypeReflectionMarker for NearClipPlane {}

impl NearClipPlane {
    /// Create a new `NearClipPlane`.
    #[inline]
    pub fn new(near_clip_plane: impl Into<crate::blueprint::components::NearClipPlane>) -> Self {
        Self {
            near_clip_plane: near_clip_plane.into(),
        }
    }
}

impl ::re_byte_size::SizeBytes for NearClipPlane {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.near_clip_plane.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <crate::blueprint::components::NearClipPlane>::is_pod()
    }
}
