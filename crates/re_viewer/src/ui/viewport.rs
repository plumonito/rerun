//! The viewport panel.
//!
//! Contains all space views.
//!
//! To do:
//! * [ ] Opening up new Space Views
//! * [ ] Controlling visibility of objects inside each Space View
//! * [ ] Transforming objects between spaces

use std::collections::BTreeMap;

use ahash::HashMap;
use itertools::Itertools as _;

use nohash_hasher::IntSet;
use re_data_store::{
    log_db::ObjDb, FieldName, ObjPath, ObjPathComp, ObjectTree, ObjectTreeProperties, Objects,
    TimeQuery, TimelineStore,
};
use re_log_types::{ObjectType, Transform, ViewCoordinates};

use crate::misc::{Selection, TimeControl, ViewerContext};

use super::view3d::SpaceCamera;

// ----------------------------------------------------------------------------

/// A unique id for each space view.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct SpaceViewId(uuid::Uuid);

impl SpaceViewId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

// ----------------------------------------------------------------------------

/// Information about one "space".
///
/// This is gathered by analyzing the transform hierarchy of the objects.
#[derive(Default)]
struct SpaceInfo {
    /// The latest known coordinate system for this space.
    coordinates: Option<ViewCoordinates>,

    /// All paths in this space (including self and children connected by the identity transform).
    objects: IntSet<ObjPath>,

    /// Nearest ancestor to whom we are not connected via an identity transform.
    #[allow(unused)] // TODO(emilk): support projecting parent space(s) into this space
    parent: Option<(ObjPath, Transform)>,

    /// Nearest descendants to whom we are not connected with an identity transform.
    child_spaces: BTreeMap<ObjPath, Transform>,
}

/// Information about all spaces.
///
/// This is gathered by analyzing the transform hierarchy of the objects.
#[derive(Default)]
struct SpacesInfo {
    spaces: BTreeMap<ObjPath, SpaceInfo>,
}

impl SpacesInfo {
    /// Do a graph analysis of the transform hierarchy, and create cuts
    /// wherever we find a non-identity transform.
    fn new(obj_db: &ObjDb, time_ctrl: &TimeControl) -> Self {
        crate::profile_function!();

        fn add_children(
            timeline_store: Option<&TimelineStore<i64>>,
            query_time: Option<i64>,
            spaces_info: &mut SpacesInfo,
            parent_space_path: &ObjPath,
            parent_space_info: &mut SpaceInfo,
            tree: &ObjectTree,
        ) {
            if let Some(transform) = query_transform(timeline_store, &tree.path, query_time) {
                // A set transform (likely non-identity) - create a new space.
                parent_space_info
                    .child_spaces
                    .insert(tree.path.clone(), transform.clone());

                let mut child_space_info = SpaceInfo {
                    parent: Some((parent_space_path.clone(), transform)),
                    ..Default::default()
                };
                child_space_info.objects.insert(tree.path.clone()); // spaces includes self

                for child_tree in tree.children.values() {
                    add_children(
                        timeline_store,
                        query_time,
                        spaces_info,
                        &tree.path,
                        &mut child_space_info,
                        child_tree,
                    );
                }
                spaces_info
                    .spaces
                    .insert(tree.path.clone(), child_space_info);
            } else {
                // no transform == identity transform.
                parent_space_info.objects.insert(tree.path.clone()); // spaces includes self

                for child_tree in tree.children.values() {
                    add_children(
                        timeline_store,
                        query_time,
                        spaces_info,
                        parent_space_path,
                        parent_space_info,
                        child_tree,
                    );
                }
            }
        }

        let timeline = time_ctrl.timeline();
        let query_time = time_ctrl.time().map(|time| time.floor().as_i64());
        let timeline_store = obj_db.store.get(timeline);

        let mut spaces_info = Self::default();

        for tree in obj_db.tree.children.values() {
            // Each root object is its own space (or should be)

            if query_transform(timeline_store, &tree.path, query_time).is_some() {
                re_log::warn_once!(
                    "Root object '{}' has a _transform - this is not allowed!",
                    tree.path
                );
            }

            let mut space_info = SpaceInfo::default();
            add_children(
                timeline_store,
                query_time,
                &mut spaces_info,
                &tree.path,
                &mut space_info,
                tree,
            );
            spaces_info.spaces.insert(tree.path.clone(), space_info);
        }

        for (obj_path, space_info) in &mut spaces_info.spaces {
            space_info.coordinates = query_view_coordinates(obj_db, time_ctrl, obj_path);
        }

        spaces_info
    }
}

// ----------------------------------------------------------------------------

/// Get the latest value of the `_transform` meta-field of the given object.
fn query_transform(
    store: Option<&TimelineStore<i64>>,
    obj_path: &ObjPath,
    query_time: Option<i64>,
) -> Option<re_log_types::Transform> {
    let field_store = store?.get(obj_path)?.get(&FieldName::from("_transform"))?;
    // `_transform` is only allowed to be stored in a mono-field.
    let mono_field_store = field_store.get_mono::<re_log_types::Transform>().ok()?;

    // There is a transform, at least at _some_ time.
    // Is there a transform _now_?
    let latest = query_time
        .and_then(|query_time| mono_field_store.latest_at(&query_time))
        .map(|(_, (_, transform))| transform.clone());

    // If not, return an unknown transform to indicate that there is
    // still a space-split.
    Some(latest.unwrap_or(Transform::Unknown))
}

/// Get the latest value of the `_view_coordinates` meta-field of the given object.
fn query_view_coordinates(
    obj_db: &ObjDb,
    time_ctrl: &TimeControl,
    obj_path: &ObjPath,
) -> Option<re_log_types::ViewCoordinates> {
    let query_time = time_ctrl.time()?;
    let timeline = time_ctrl.timeline();

    let store = obj_db.store.get(timeline)?;

    let field_store = store
        .get(obj_path)?
        .get(&re_data_store::FieldName::from("_view_coordinates"))?;

    // `_view_coordinates` is only allowed to be stored in a mono-field.
    let mono_field_store = field_store
        .get_mono::<re_log_types::ViewCoordinates>()
        .ok()?;

    mono_field_store
        .latest_at(&query_time.floor().as_i64())
        .map(|(_time, (_msg_id, system))| *system)
}

// ----------------------------------------------------------------------------

type VisibilitySet = BTreeMap<SpaceViewId, bool>;

/// Describes the layout and contents of the Viewport Panel.
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ViewportBlueprint {
    /// Where the space views are stored.
    space_views: HashMap<SpaceViewId, SpaceView>,

    /// Which views are visible.
    visible: VisibilitySet,

    /// The layouts of all the space views.
    ///
    /// One for each combination of what views are visible.
    /// So if a user toggles the visibility of one SpaceView, we
    /// switch which layout we are using. This is somewhat hacky.
    trees: HashMap<VisibilitySet, egui_dock::Tree<SpaceViewId>>,

    /// Show one tab as maximized?
    maximized: Option<SpaceViewId>,
}

impl ViewportBlueprint {
    /// Create a default suggested blueprint using some heuristics.
    fn new(obj_db: &ObjDb, spaces_info: &SpacesInfo) -> Self {
        crate::profile_function!();

        let mut blueprint = Self::default();

        for (path, space_info) in &spaces_info.spaces {
            if should_have_default_view(obj_db, space_info) {
                let space_view_id = SpaceViewId::random();

                blueprint
                    .space_views
                    .insert(space_view_id, SpaceView::from_path(path.clone()));

                blueprint.visible.insert(space_view_id, true);
            }
        }

        blueprint
    }

    pub fn get_space_view_mut(&mut self, space_view: &SpaceViewId) -> Option<&mut SpaceView> {
        self.space_views.get_mut(space_view)
    }

    fn has_space(&self, space_path: &ObjPath) -> bool {
        self.space_views
            .values()
            .any(|view| &view.space_path == space_path)
    }

    /// Show the blueprint panel tree view.
    fn tree_ui(
        &mut self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        spaces_info: &SpacesInfo,
        obj_tree: &ObjectTree,
    ) {
        crate::profile_function!();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let space_view_ids = self
                    .space_views
                    .keys()
                    .sorted_by_key(|space_view_id| {
                        &self.space_views.get(space_view_id).unwrap().name
                    })
                    .copied()
                    .collect_vec();

                for space_view_id in &space_view_ids {
                    self.space_view_tree_ui(ctx, ui, spaces_info, obj_tree, space_view_id);
                }
            });
    }

    fn space_view_tree_ui(
        &mut self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        spaces_info: &SpacesInfo,
        obj_tree: &ObjectTree,
        space_view_id: &SpaceViewId,
    ) {
        let space_view = self.space_views.get_mut(space_view_id).unwrap();

        let space_path = &space_view.space_path;
        let collapsing_header_id = ui.make_persistent_id(&space_view_id);
        let default_open = true;
        egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            collapsing_header_id,
            default_open,
        )
        .show_header(ui, |ui| {
            ui.label("🗖"); // icon indicating this is a space-view

            let is_selected = ctx.rec_cfg.selection == Selection::SpaceView(*space_view_id);
            if ui.selectable_label(is_selected, &space_view.name).clicked() {
                ctx.rec_cfg.selection = Selection::SpaceView(*space_view_id);
                if let Some(tree) = self.trees.get_mut(&self.visible) {
                    focus_tab(tree, space_view_id);
                }
            }

            let is_space_view_visible = self.visible.entry(*space_view_id).or_insert(true);
            visibility_button(ui, true, is_space_view_visible);
        })
        .body(|ui| {
            if let Some(space_info) = spaces_info.spaces.get(space_path) {
                if let Some(tree) = obj_tree.subtree(space_path) {
                    let is_space_view_visible = self.visible.entry(*space_view_id).or_insert(true);
                    show_obj_tree_children(
                        ctx,
                        ui,
                        *is_space_view_visible,
                        &mut space_view.obj_tree_properties,
                        space_info,
                        tree,
                    );
                }
            }
        });
    }

    fn add_space_view(&mut self, path: &ObjPath) {
        let space_view_id = SpaceViewId::random();

        self.space_views
            .insert(space_view_id, SpaceView::from_path(path.clone()));

        self.visible.insert(space_view_id, true);

        self.trees.clear(); // Reset them
    }
}

/// Is this space worthy of its on space view by default?
fn should_have_default_view(obj_db: &ObjDb, space_info: &SpaceInfo) -> bool {
    if space_info.objects.len() == 1 {
        // Only one object in this view…
        let obj = space_info.objects.iter().next().unwrap();
        if obj_db.types.get(obj.obj_type_path()).is_none() {
            return false; // It doesn't have a type, so it is probably just the `_transform`, so nothing to show.
        }
    }
    true
}

fn show_obj_tree(
    ctx: &mut ViewerContext<'_>,
    ui: &mut egui::Ui,
    parent_is_visible: bool,
    obj_tree_properties: &mut ObjectTreeProperties,
    space_info: &SpaceInfo,
    name: String,
    tree: &ObjectTree,
) {
    if tree.is_leaf() {
        ui.horizontal(|ui| {
            ctx.obj_path_button_to(ui, name, &tree.path);
            object_visibility_button(ui, parent_is_visible, obj_tree_properties, &tree.path);
        });
    } else {
        let collapsing_header_id = ui.id().with(&tree.path);
        let default_open = false;
        egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            collapsing_header_id,
            default_open,
        )
        .show_header(ui, |ui| {
            ctx.obj_path_button_to(ui, name, &tree.path);
            object_visibility_button(ui, parent_is_visible, obj_tree_properties, &tree.path);
        })
        .body(|ui| {
            show_obj_tree_children(
                ctx,
                ui,
                parent_is_visible,
                obj_tree_properties,
                space_info,
                tree,
            );
        });
    }
}

fn show_obj_tree_children(
    ctx: &mut ViewerContext<'_>,
    ui: &mut egui::Ui,
    parent_is_visible: bool,
    obj_tree_properties: &mut ObjectTreeProperties,
    space_info: &SpaceInfo,
    tree: &ObjectTree,
) {
    for (path_comp, child) in &tree.children {
        if space_info.objects.contains(&child.path) {
            show_obj_tree(
                ctx,
                ui,
                parent_is_visible,
                obj_tree_properties,
                space_info,
                path_comp.to_string(),
                child,
            );
        }
    }
}

fn object_visibility_button(
    ui: &mut egui::Ui,
    parent_is_visible: bool,
    obj_tree_properties: &mut ObjectTreeProperties,
    path: &ObjPath,
) {
    let are_all_ancestors_visible = parent_is_visible
        && match path.parent() {
            None => true, // root
            Some(parent) => obj_tree_properties.projected.get(&parent).visible,
        };

    let mut props = obj_tree_properties.individual.get(path);

    if visibility_button(ui, are_all_ancestors_visible, &mut props.visible).changed() {
        obj_tree_properties.individual.set(path.clone(), props);
    }
}

fn visibility_button(ui: &mut egui::Ui, enabled: bool, visible: &mut bool) -> egui::Response {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.set_enabled(enabled);
        if enabled {
            ui.toggle_value(visible, "👁")
        } else {
            let mut always_false = false;
            ui.toggle_value(&mut always_false, "👁")
        }
        .on_hover_text("Toggle visibility")
    })
    .inner
}

// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum ViewCategory {
    TwoD,
    #[default]
    ThreeD,
    Tensor,
    Text,
}

// ----------------------------------------------------------------------------

/// A view of a space.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SpaceView {
    pub name: String,
    pub space_path: ObjPath,
    view_state: ViewState,

    /// In case we are a mix of 2d/3d/tensor/text, we show what?
    selected_category: ViewCategory,

    obj_tree_properties: ObjectTreeProperties,
}

impl SpaceView {
    fn from_path(space_path: ObjPath) -> Self {
        Self {
            name: space_path.to_string(),
            space_path,
            view_state: Default::default(),
            selected_category: Default::default(),
            obj_tree_properties: Default::default(),
        }
    }

    fn on_frame_start(&mut self, obj_tree: &ObjectTree) {
        self.obj_tree_properties.on_frame_start(obj_tree);
    }

    fn objects_ui(
        &mut self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        spaces_info: &SpacesInfo,
        space_info: &SpaceInfo,
        time_objects: &Objects<'_>,
        sticky_objects: &Objects<'_>,
    ) -> egui::Response {
        crate::profile_function!();

        let multidim_tensor = multidim_tensor(time_objects);
        let has_2d =
            time_objects.has_any_2d() && (multidim_tensor.is_none() || time_objects.len() > 1);
        let has_3d = time_objects.has_any_3d();
        let has_text = sticky_objects.has_any_text_entries();

        let mut categories = vec![];
        if has_2d {
            categories.push(ViewCategory::TwoD);
        }
        if has_3d {
            categories.push(ViewCategory::ThreeD);
        }
        if multidim_tensor.is_some() {
            categories.push(ViewCategory::Tensor);
        }
        if has_text {
            categories.push(ViewCategory::Text);
        }

        match categories.len() {
            0 => ui.label("(empty)"),
            1 => {
                if has_2d {
                    self.view_state
                        .ui_2d(ctx, ui, &self.space_path, time_objects)
                } else if has_3d {
                    self.view_state.ui_3d(
                        ctx,
                        ui,
                        &self.space_path,
                        spaces_info,
                        space_info,
                        time_objects,
                    )
                } else if let Some(multidim_tensor) = multidim_tensor {
                    self.view_state.ui_tensor(ui, multidim_tensor)
                } else {
                    self.view_state.ui_text(ctx, ui, sticky_objects)
                }
            }
            _ => {
                // Show tabs to let user select which category to view
                ui.vertical(|ui| {
                    if !categories.contains(&self.selected_category) {
                        self.selected_category = categories[0];
                    }

                    ui.horizontal(|ui| {
                        for category in categories {
                            let text = match category {
                                ViewCategory::TwoD => "2D",
                                ViewCategory::ThreeD => "3D",
                                ViewCategory::Tensor => "Tensor",
                                ViewCategory::Text => "Text",
                            };
                            ui.selectable_value(&mut self.selected_category, category, text);
                            // TODO(emilk): make it look like tabs
                        }
                    });
                    ui.separator();

                    match self.selected_category {
                        ViewCategory::TwoD => {
                            self.view_state
                                .ui_2d(ctx, ui, &self.space_path, time_objects);
                        }
                        ViewCategory::ThreeD => {
                            self.view_state.ui_3d(
                                ctx,
                                ui,
                                &self.space_path,
                                spaces_info,
                                space_info,
                                time_objects,
                            );
                        }
                        ViewCategory::Tensor => {
                            self.view_state.ui_tensor(ui, multidim_tensor.unwrap());
                        }
                        ViewCategory::Text => {
                            self.view_state.ui_text(ctx, ui, sticky_objects);
                        }
                    }
                })
                .response
            }
        }
    }
}

fn is_sticky_type(obj_type: &ObjectType) -> bool {
    obj_type == &ObjectType::TextEntry
}

// ----------------------------------------------------------------------------

/// Camera position and similar.
#[derive(Default, serde::Deserialize, serde::Serialize)]
struct ViewState {
    // per space
    state_2d: crate::view2d::State2D,

    state_3d: crate::view3d::State3D,

    state_tensor: Option<crate::view_tensor::TensorViewState>,

    state_text_entry: crate::text_entry_view::TextEntryState,
}

impl ViewState {
    fn ui_2d(
        &mut self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        space: &ObjPath,
        objects: &Objects<'_>,
    ) -> egui::Response {
        crate::view2d::view_2d(ctx, ui, &mut self.state_2d, Some(space), objects)
    }

    fn ui_3d(
        &mut self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        space: &ObjPath,
        spaces_info: &SpacesInfo,
        space_info: &SpaceInfo,
        objects: &Objects<'_>,
    ) -> egui::Response {
        ui.vertical(|ui| {
            let state = &mut self.state_3d;
            let space_cameras = &space_cameras(spaces_info, space_info);
            let coordinates = space_info.coordinates;
            let space_specs = crate::view3d::SpaceSpecs::from_view_coordinates(coordinates);
            let scene = crate::view3d::scene::Scene::from_objects(ctx, objects);
            crate::view3d::view_3d(
                ctx,
                ui,
                state,
                Some(space),
                &space_specs,
                scene,
                space_cameras,
            );
        })
        .response
    }

    fn ui_tensor(&mut self, ui: &mut egui::Ui, tensor: &re_log_types::Tensor) -> egui::Response {
        let state_tensor = self
            .state_tensor
            .get_or_insert_with(|| crate::ui::view_tensor::TensorViewState::create(tensor));
        ui.vertical(|ui| {
            crate::view_tensor::view_tensor(ui, state_tensor, tensor);
        })
        .response
    }

    fn ui_text(
        &mut self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        objects: &Objects<'_>,
    ) -> egui::Response {
        self.state_text_entry.show(ui, ctx, objects)
    }
}

/// Look for camera transform and pinhole in the transform hierarchy
/// and return them as cameras.
fn space_cameras(spaces_info: &SpacesInfo, space_info: &SpaceInfo) -> Vec<SpaceCamera> {
    crate::profile_function!();

    let mut space_cameras = vec![];

    for (child_path, child_transform) in &space_info.child_spaces {
        if let Transform::Rigid3(world_from_camera) = child_transform {
            let world_from_camera = world_from_camera.parent_from_child();

            let view_space = spaces_info
                .spaces
                .get(child_path)
                .and_then(|child| child.coordinates);

            let mut found_any_pinhole = false;

            if let Some(child_space_info) = spaces_info.spaces.get(child_path) {
                for (grand_child_path, grand_child_transform) in &child_space_info.child_spaces {
                    if let Transform::Pinhole(pinhole) = grand_child_transform {
                        space_cameras.push(SpaceCamera {
                            camera_obj_path: child_path.clone(),
                            instance_index_hash: re_log_types::IndexHash::NONE,
                            camera_view_coordinates: view_space,
                            world_from_camera,
                            pinhole: Some(*pinhole),
                            target_space: Some(grand_child_path.clone()),
                        });
                        found_any_pinhole = true;
                    }
                }
            }

            if !found_any_pinhole {
                space_cameras.push(SpaceCamera {
                    camera_obj_path: child_path.clone(),
                    instance_index_hash: re_log_types::IndexHash::NONE,
                    camera_view_coordinates: view_space,
                    world_from_camera,
                    pinhole: None,
                    target_space: None,
                });
            }
        }
    }

    space_cameras
}

fn multidim_tensor<'s>(objects: &Objects<'s>) -> Option<&'s re_log_types::Tensor> {
    // We have a special tensor viewer that (currently) only works
    // when we only have a single tensor (and no bounding boxes etc).
    // It is also not as great for images as the normal 2d view (at least not yet).
    // This is a hacky-way of detecting this special case.
    // TODO(emilk): integrate the tensor viewer into the 2D viewer instead,
    // so we can stack bounding boxes etc on top of it.
    if objects.image.len() == 1 {
        let image = objects.image.first().unwrap().1;
        let tensor = image.tensor;

        // Ignore tensors that likely represent images.
        if tensor.num_dim() > 3 || tensor.num_dim() == 3 && tensor.shape.last().unwrap().size > 4 {
            return Some(tensor);
        }
    }
    None
}

// ----------------------------------------------------------------------------

struct TabViewer<'a, 'b> {
    ctx: &'a mut ViewerContext<'b>,
    spaces_info: &'a SpacesInfo,
    space_views: &'a mut HashMap<SpaceViewId, SpaceView>,
    maximized: &'a mut Option<SpaceViewId>,
}

impl<'a, 'b> egui_dock::TabViewer for TabViewer<'a, 'b> {
    type Tab = SpaceViewId;

    fn ui(&mut self, ui: &mut egui::Ui, space_view_id: &mut Self::Tab) {
        crate::profile_function!();

        ui.horizontal_top(|ui| {
            if ui.button("🗖").on_hover_text("Maximize space").clicked() {
                *self.maximized = Some(*space_view_id);
            }

            let space_view = self
                .space_views
                .get_mut(space_view_id)
                .expect("Should have been populated beforehand");

            space_view_ui(self.ctx, ui, self.spaces_info, space_view);
        });
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let space_view = self
            .space_views
            .get_mut(tab)
            .expect("Should have been populated beforehand");
        space_view.name.clone().into()
    }
}

fn space_view_ui(
    ctx: &mut ViewerContext<'_>,
    ui: &mut egui::Ui,
    spaces_info: &SpacesInfo,
    space_view: &mut SpaceView,
) -> egui::Response {
    if let Some(space_info) = spaces_info.spaces.get(&space_view.space_path) {
        let obj_tree_properties = &space_view.obj_tree_properties;

        // Get the latest objects for the currently selected time:
        let mut time_objects = Objects::default();
        {
            crate::profile_scope!("time_query");
            let timeline = ctx.rec_cfg.time_ctrl.timeline();
            if let Some(timeline_store) = ctx.log_db.obj_db.store.get(timeline) {
                if let Some(time_query) = ctx.rec_cfg.time_ctrl.time_query() {
                    for obj_path in &space_info.objects {
                        if obj_tree_properties.projected.get(obj_path).visible {
                            if let Some(obj_store) = timeline_store.get(obj_path) {
                                if let Some(obj_type) =
                                    ctx.log_db.obj_db.types.get(obj_path.obj_type_path())
                                {
                                    if !is_sticky_type(obj_type) {
                                        time_objects.query_object(
                                            obj_store,
                                            &time_query,
                                            obj_path,
                                            obj_type,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let time_objects = filter_objects(&time_objects);

        // Get the "sticky" objects (e.g. text logs)
        // that don't care about the current time:
        let mut sticky_objects = Objects::default();
        {
            crate::profile_scope!("sticky_query");
            let timeline = ctx.rec_cfg.time_ctrl.timeline();
            if let Some(timeline_store) = ctx.log_db.obj_db.store.get(timeline) {
                for obj_path in &space_info.objects {
                    if obj_tree_properties.projected.get(obj_path).visible {
                        if let Some(obj_store) = timeline_store.get(obj_path) {
                            if let Some(obj_type) =
                                ctx.log_db.obj_db.types.get(obj_path.obj_type_path())
                            {
                                if is_sticky_type(obj_type) {
                                    sticky_objects.query_object(
                                        obj_store,
                                        &TimeQuery::EVERYTHING,
                                        obj_path,
                                        obj_type,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        let sticky_objects = filter_objects(&sticky_objects);

        space_view.objects_ui(
            ctx,
            ui,
            spaces_info,
            space_info,
            &time_objects,
            &sticky_objects,
        )
    } else {
        unknown_space_label(ui, &space_view.space_path)
    }
}

fn filter_objects<'s>(objects: &'_ Objects<'s>) -> Objects<'s> {
    crate::profile_function!();
    objects.filter(|props| props.visible)
}

fn unknown_space_label(ui: &mut egui::Ui, space_path: &ObjPath) -> egui::Response {
    ui.colored_label(
        ui.visuals().warn_fg_color,
        format!("Unknown space {space_path}"),
    )
}

// ----------------------------------------------------------------------------

/// Defines the layout of the whole Viewer (or will, eventually).
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Blueprint {
    pub viewport: ViewportBlueprint,
}

impl Blueprint {
    pub fn blueprint_panel_and_viewport(&mut self, ctx: &mut ViewerContext<'_>, ui: &mut egui::Ui) {
        crate::profile_function!();

        let spaces_info = SpacesInfo::new(&ctx.log_db.obj_db, &ctx.rec_cfg.time_ctrl);

        self.viewport.on_frame_start(ctx, &spaces_info);
        self.viewport.blueprint_panel(ctx, ui, &spaces_info);

        let viewport_frame = egui::Frame {
            fill: ui.style().visuals.window_fill(),
            ..Default::default()
        };

        egui::CentralPanel::default()
            .frame(viewport_frame)
            .show_inside(ui, |ui| {
                self.viewport.viewport_ui(ui, ctx, &spaces_info);
            });
    }
}

impl ViewportBlueprint {
    fn on_frame_start(&mut self, ctx: &mut ViewerContext<'_>, spaces_info: &SpacesInfo) {
        crate::profile_function!();

        if self.space_views.is_empty() {
            *self = Self::new(&ctx.log_db.obj_db, spaces_info);
        } else {
            // Check if the blueprint is missing a space,
            // maybe one that has been added by new data:
            for (path, space_info) in &spaces_info.spaces {
                if should_have_default_view(&ctx.log_db.obj_db, space_info) && !self.has_space(path)
                {
                    self.add_space_view(path);
                }
            }
        }

        for space_view in self.space_views.values_mut() {
            space_view.on_frame_start(&ctx.log_db.obj_db.tree);
        }
    }

    fn blueprint_panel(
        &mut self,
        ctx: &mut ViewerContext<'_>,
        ui: &mut egui::Ui,
        spaces_info: &SpacesInfo,
    ) {
        let side_panel_frame = egui::Frame {
            fill: ui.style().visuals.window_fill(),
            inner_margin: egui::style::Margin::same(4.0),
            stroke: ui.style().visuals.window_stroke(),
            ..Default::default()
        };

        egui::SidePanel::left("blueprint_panel")
            .resizable(true)
            .frame(side_panel_frame)
            .default_width(200.0)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.button("Reset space views").clicked() {
                        *self = Self::new(&ctx.log_db.obj_db, spaces_info);
                    }
                });

                ui.separator();

                self.tree_ui(ctx, ui, spaces_info, &ctx.log_db.obj_db.tree);
            });
    }

    fn viewport_ui(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut ViewerContext<'_>,
        spaces_info: &SpacesInfo,
    ) {
        // Lazily create a layout tree based on which SpaceViews are currently visible:
        let tree = self.trees.entry(self.visible.clone()).or_insert_with(|| {
            tree_from_space_views(ui.available_size(), &self.visible, &self.space_views)
        });

        let num_space_views = num_tabs(tree);
        if num_space_views == 0 {
            // nothing to show
        } else if num_space_views == 1 {
            let space_view_id = first_tab(tree).unwrap();
            let space_view = self
                .space_views
                .get_mut(&space_view_id)
                .expect("Should have been populated beforehand");

            ui.strong(&space_view.name);

            space_view_ui(ctx, ui, spaces_info, space_view);
        } else if let Some(space_view_id) = self.maximized {
            let space_view = self
                .space_views
                .get_mut(&space_view_id)
                .expect("Should have been populated beforehand");

            ui.horizontal(|ui| {
                if ui
                    .button("⬅")
                    .on_hover_text("Restore - show all spaces")
                    .clicked()
                {
                    self.maximized = None;
                }
                ui.strong(&space_view.name);
            });

            space_view_ui(ctx, ui, spaces_info, space_view);
        } else {
            let mut dock_style = egui_dock::Style::from_egui(ui.style().as_ref());
            dock_style.separator_width = 2.0;
            dock_style.show_close_buttons = false;
            dock_style.tab_include_scrollarea = false;

            let mut tab_viewer = TabViewer {
                ctx,
                spaces_info,
                space_views: &mut self.space_views,
                maximized: &mut self.maximized,
            };

            egui_dock::DockArea::new(tree)
                .style(dock_style)
                .show_inside(ui, &mut tab_viewer);
        }
    }
}

// ----------------------------------------------------------------------------
// Code for automatic layout of panels:

fn tree_from_space_views(
    available_size: egui::Vec2,
    visible: &BTreeMap<SpaceViewId, bool>,
    space_views: &HashMap<SpaceViewId, SpaceView>,
) -> egui_dock::Tree<SpaceViewId> {
    let mut tree = egui_dock::Tree::new(vec![]);

    let mut space_make_infos = space_views
        .iter()
        .filter(|(space_view_id, _space_view)| {
            visible.get(space_view_id).copied().unwrap_or_default()
        })
        .map(|(space_view_id, space_view)| {
            SpaceMakeInfo {
                id: *space_view_id,
                path: space_view.space_path.clone(),
                size2d: None, // TODO(emilk): figure out the size of spaces somehow. Each object path could have a running bbox?
            }
        })
        .collect_vec();

    if !space_make_infos.is_empty() {
        let layout = layout_spaces(available_size, &mut space_make_infos);
        tree_from_split(&mut tree, egui_dock::NodeIndex(0), &layout);
    }

    tree
}

#[derive(Clone, Debug)]
struct SpaceMakeInfo {
    id: SpaceViewId,
    path: ObjPath,
    size2d: Option<egui::Vec2>,
}

impl SpaceMakeInfo {
    fn is_2d(&self) -> bool {
        self.size2d.is_some()
    }
}

enum LayoutSplit {
    LeftRight(Box<LayoutSplit>, f32, Box<LayoutSplit>),
    TopBottom(Box<LayoutSplit>, f32, Box<LayoutSplit>),
    Leaf(SpaceMakeInfo),
}

fn tree_from_split(
    tree: &mut egui_dock::Tree<SpaceViewId>,
    parent: egui_dock::NodeIndex,
    split: &LayoutSplit,
) {
    match split {
        LayoutSplit::LeftRight(left, fraction, right) => {
            let [left_ni, right_ni] = tree.split_right(parent, *fraction, vec![]);
            tree_from_split(tree, left_ni, left);
            tree_from_split(tree, right_ni, right);
        }
        LayoutSplit::TopBottom(top, fraction, bottom) => {
            let [top_ni, bottom_ni] = tree.split_below(parent, *fraction, vec![]);
            tree_from_split(tree, top_ni, top);
            tree_from_split(tree, bottom_ni, bottom);
        }
        LayoutSplit::Leaf(space_info) => {
            tree.set_focused_node(parent);
            tree.push_to_focused_leaf(space_info.id);
        }
    }
}

// TODO(emilk): fix O(N^2) execution for layout_spaces
fn layout_spaces(size: egui::Vec2, spaces: &mut [SpaceMakeInfo]) -> LayoutSplit {
    assert!(!spaces.is_empty());

    if spaces.len() == 1 {
        LayoutSplit::Leaf(spaces[0].clone())
    } else {
        spaces.sort_by_key(|si| si.is_2d());
        let start_3d = spaces.partition_point(|si| !si.is_2d());

        if 0 < start_3d && start_3d < spaces.len() {
            split_spaces_at(size, spaces, start_3d)
        } else {
            // All 2D or all 3D
            let groups = group_by_path_prefix(spaces);
            assert!(groups.len() > 1);

            let num_spaces = spaces.len();

            let mut best_split = 0;
            let mut rearranged_spaces = vec![];
            for mut group in groups {
                rearranged_spaces.append(&mut group);

                let split_candidate = rearranged_spaces.len();
                if (split_candidate as f32 / num_spaces as f32 - 0.5).abs()
                    < (best_split as f32 / num_spaces as f32 - 0.5).abs()
                {
                    best_split = split_candidate;
                }
            }
            assert_eq!(rearranged_spaces.len(), num_spaces);
            assert!(0 < best_split && best_split < num_spaces,);

            split_spaces_at(size, &mut rearranged_spaces, best_split)
        }
    }
}

fn split_spaces_at(size: egui::Vec2, spaces: &mut [SpaceMakeInfo], index: usize) -> LayoutSplit {
    use egui::vec2;

    assert!(0 < index && index < spaces.len());

    let t = index as f32 / spaces.len() as f32;
    let desired_aspect_ratio = desired_aspect_ratio(spaces).unwrap_or(16.0 / 9.0);

    if size.x > desired_aspect_ratio * size.y {
        let left = layout_spaces(vec2(size.x * t, size.y), &mut spaces[..index]);
        let right = layout_spaces(vec2(size.x * (1.0 - t), size.y), &mut spaces[index..]);
        LayoutSplit::LeftRight(left.into(), t, right.into())
    } else {
        let top = layout_spaces(vec2(size.y, size.y * t), &mut spaces[..index]);
        let bottom = layout_spaces(vec2(size.y, size.x * (1.0 - t)), &mut spaces[index..]);
        LayoutSplit::TopBottom(top.into(), t, bottom.into())
    }
}

fn desired_aspect_ratio(spaces: &[SpaceMakeInfo]) -> Option<f32> {
    let mut sum = 0.0;
    let mut num = 0.0;
    for space in spaces {
        if let Some(size) = space.size2d {
            let aspect = size.x / size.y;
            if aspect.is_finite() {
                sum += aspect;
                num += 1.0;
            }
        }
    }

    (num != 0.0).then_some(sum / num)
}

fn group_by_path_prefix(space_infos: &[SpaceMakeInfo]) -> Vec<Vec<SpaceMakeInfo>> {
    if space_infos.len() < 2 {
        return vec![space_infos.to_vec()];
    }
    crate::profile_function!();

    let paths = space_infos
        .iter()
        .map(|space_info| space_info.path.to_components())
        .collect_vec();

    for i in 0.. {
        let mut groups: std::collections::BTreeMap<Option<&ObjPathComp>, Vec<&SpaceMakeInfo>> =
            Default::default();
        for (path, space) in paths.iter().zip(space_infos) {
            groups.entry(path.get(i)).or_default().push(space);
        }
        if groups.len() == 1 && groups.contains_key(&None) {
            break;
        }
        if groups.len() > 1 {
            return groups
                .values()
                .map(|spaces| spaces.iter().cloned().cloned().collect())
                .collect();
        }
    }
    space_infos
        .iter()
        .map(|space| vec![space.clone()])
        .collect()
}

// ----------------------------------------------------------------------------

// TODO(emilk): replace with https://github.com/Adanos020/egui_dock/pull/53 when we update egui_dock
fn num_tabs(tree: &egui_dock::Tree<SpaceViewId>) -> usize {
    let mut count = 0;
    for node in tree.iter() {
        if let egui_dock::Node::Leaf { tabs, .. } = node {
            count += tabs.len();
        }
    }
    count
}

// TODO(emilk): replace with https://github.com/Adanos020/egui_dock/pull/53 when we update egui_dock
fn first_tab(tree: &egui_dock::Tree<SpaceViewId>) -> Option<SpaceViewId> {
    for node in tree.iter() {
        if let egui_dock::Node::Leaf { tabs, .. } = node {
            if let Some(first) = tabs.first() {
                return Some(*first);
            }
        }
    }
    None
}

fn focus_tab(tree: &mut egui_dock::Tree<SpaceViewId>, tab: &SpaceViewId) {
    if let Some((node_index, tab_index)) = tree.find_tab(tab) {
        tree.set_focused_node(node_index);
        tree.set_active_tab(node_index, tab_index);
    }
}
