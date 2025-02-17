// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/store/re_types/definitions/rerun/archetypes/instance_poses3d.fbs".

#pragma once

#include "../collection.hpp"
#include "../compiler_utils.hpp"
#include "../component_batch.hpp"
#include "../components/pose_rotation_axis_angle.hpp"
#include "../components/pose_rotation_quat.hpp"
#include "../components/pose_scale3d.hpp"
#include "../components/pose_transform_mat3x3.hpp"
#include "../components/pose_translation3d.hpp"
#include "../indicator_component.hpp"
#include "../result.hpp"

#include <cstdint>
#include <optional>
#include <utility>
#include <vector>

namespace rerun::archetypes {
    /// **Archetype**: One or more transforms between the current entity and its parent. Unlike `archetypes::Transform3D`, it is *not* propagated in the transform hierarchy.
    ///
    /// If both `archetypes::InstancePoses3D` and `archetypes::Transform3D` are present,
    /// first the tree propagating `archetypes::Transform3D` is applied, then `archetypes::InstancePoses3D`.
    ///
    /// From the point of view of the entity's coordinate system,
    /// all components are applied in the inverse order they are listed here.
    /// E.g. if both a translation and a max3x3 transform are present,
    /// the 3x3 matrix is applied first, followed by the translation.
    ///
    /// Currently, many visualizers support only a single instance transform per entity.
    /// Check archetype documentations for details - if not otherwise specified, only the first instance transform is applied.
    /// Some visualizers like the mesh visualizer used for `archetypes::Mesh3D`,
    /// will draw an object for every pose, a behavior also known as "instancing".
    ///
    /// ## Example
    ///
    /// ### Regular & instance transforms in tandem
    /// ![image](https://static.rerun.io/leaf_transform3d/41674f0082d6de489f8a1cd1583f60f6b5820ddf/full.png)
    ///
    /// ```cpp
    /// #include <rerun.hpp>
    /// #include <rerun/demo_utils.hpp>
    ///
    /// int main() {
    ///     const auto rec = rerun::RecordingStream("rerun_example_instance_pose3d_combined");
    ///     rec.set_time_sequence("frame", 0);
    ///
    ///     // Log a box and points further down in the hierarchy.
    ///     rec.log("world/box", rerun::Boxes3D::from_half_sizes({{1.0, 1.0, 1.0}}));
    ///     rec.log(
    ///         "world/box/points",
    ///         rerun::Points3D(rerun::demo::grid3d<rerun::Position3D, float>(-10.0f, 10.0f, 10))
    ///     );
    ///
    ///     for (int i = 0; i <180; ++i) {
    ///         rec.set_time_sequence("frame", i);
    ///
    ///         // Log a regular transform which affects both the box and the points.
    ///         rec.log(
    ///             "world/box",
    ///             rerun::Transform3D::from_rotation(rerun::RotationAxisAngle{
    ///                 {0.0f, 0.0f, 1.0f},
    ///                 rerun::Angle::degrees(static_cast<float>(i) * 2.0f)})
    ///         );
    ///
    ///         // Log an instance pose which affects only the box.
    ///         rec.log(
    ///             "world/box",
    ///             rerun::InstancePoses3D().with_translations(
    ///                 {{0.0f, 0.0f, std::abs(static_cast<float>(i) * 0.1f - 5.0f) - 5.0f}}
    ///             )
    ///         );
    ///     }
    /// }
    /// ```
    struct InstancePoses3D {
        /// Translation vectors.
        std::optional<Collection<rerun::components::PoseTranslation3D>> translations;

        /// Rotations via axis + angle.
        std::optional<Collection<rerun::components::PoseRotationAxisAngle>> rotation_axis_angles;

        /// Rotations via quaternion.
        std::optional<Collection<rerun::components::PoseRotationQuat>> quaternions;

        /// Scaling factors.
        std::optional<Collection<rerun::components::PoseScale3D>> scales;

        /// 3x3 transformation matrices.
        std::optional<Collection<rerun::components::PoseTransformMat3x3>> mat3x3;

      public:
        static constexpr const char IndicatorComponentName[] =
            "rerun.components.InstancePoses3DIndicator";

        /// Indicator component, used to identify the archetype when converting to a list of components.
        using IndicatorComponent = rerun::components::IndicatorComponent<IndicatorComponentName>;

      public:
        InstancePoses3D() = default;
        InstancePoses3D(InstancePoses3D&& other) = default;

        /// Translation vectors.
        InstancePoses3D with_translations(
            Collection<rerun::components::PoseTranslation3D> _translations
        ) && {
            translations = std::move(_translations);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// Rotations via axis + angle.
        InstancePoses3D with_rotation_axis_angles(
            Collection<rerun::components::PoseRotationAxisAngle> _rotation_axis_angles
        ) && {
            rotation_axis_angles = std::move(_rotation_axis_angles);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// Rotations via quaternion.
        InstancePoses3D with_quaternions(
            Collection<rerun::components::PoseRotationQuat> _quaternions
        ) && {
            quaternions = std::move(_quaternions);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// Scaling factors.
        InstancePoses3D with_scales(Collection<rerun::components::PoseScale3D> _scales) && {
            scales = std::move(_scales);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }

        /// 3x3 transformation matrices.
        InstancePoses3D with_mat3x3(Collection<rerun::components::PoseTransformMat3x3> _mat3x3) && {
            mat3x3 = std::move(_mat3x3);
            // See: https://github.com/rerun-io/rerun/issues/4027
            RR_WITH_MAYBE_UNINITIALIZED_DISABLED(return std::move(*this);)
        }
    };

} // namespace rerun::archetypes

namespace rerun {
    /// \private
    template <typename T>
    struct AsComponents;

    /// \private
    template <>
    struct AsComponents<archetypes::InstancePoses3D> {
        /// Serialize all set component batches.
        static Result<std::vector<ComponentBatch>> serialize(
            const archetypes::InstancePoses3D& archetype
        );
    };
} // namespace rerun
