use crate::ScreenStyle;
use bevy::{
  app::{HierarchyPropagatePlugin, Propagate},
  camera::{Camera3dDepthLoadOp, CameraUpdateSystems, Hdr, visibility::RenderLayers},
  core_pipeline::tonemapping::{DebandDither, Tonemapping},
  ecs::system::EntityCommands,
  prelude::*,
};

const UNSTYLED_LAYER: usize = 1;

pub fn overlay_plugin(app: &mut App) {
  app
    .add_plugins(HierarchyPropagatePlugin::<RenderLayers>::new(PostUpdate))
    .add_systems(PostUpdate, follow_camera.before(CameraUpdateSystems))
    .add_observer(on_screen_style_added);
}

#[derive(Component, Debug, Default)]
#[require(Propagate<RenderLayers> = Propagate(RenderLayers::layer(UNSTYLED_LAYER)))]
pub struct Unstyled;

#[derive(Component, Debug, Default)]
#[require(RenderLayers = RenderLayers::default().with(UNSTYLED_LAYER))]
pub struct UnstyledLight;

#[derive(Component, Debug, Default)]
struct OverlayCamera;

fn on_screen_style_added(added: On<Add<ScreenStyle>>, mut commands: Commands) {
  commands.spawn((
    Name::new("Unstyled Overlay Camera"),
    OverlayCamera,
    Camera3d {
      depth_load_op: Camera3dDepthLoadOp::Load,
      ..default()
    },
    Camera {
      order: 1,
      clear_color: ClearColorConfig::None,
      ..default()
    },
    Tonemapping::None,
    DebandDither::Disabled,
    RenderLayers::layer(UNSTYLED_LAYER),
    ChildOf(added.entity),
  ));
}

fn follow_camera(
  cameras: Query<(Ref<Projection>, &Msaa, Has<Hdr>, &Children), With<ScreenStyle>>,
  mut overlays: Query<
    (Entity, &mut Projection, &mut Msaa, Has<Hdr>),
    (With<OverlayCamera>, Without<ScreenStyle>),
  >,
  mut commands: Commands,
) {
  for (projection, msaa, is_hdr, children) in &cameras {
    let mut overlays = overlays.iter_many_mut(children).matched();
    while let Some((overlay, overlay_projection, mut overlay_msaa, overlay_is_hdr)) =
      overlays.fetch_next()
    {
      copy_projection(&projection, overlay_projection);
      overlay_msaa.set_if_neq(*msaa);
      copy_hdr(is_hdr, overlay_is_hdr, commands.entity(overlay));
    }
  }
}

fn copy_projection(styled: &Ref<Projection>, mut overlay: Mut<Projection>) {
  if styled.is_changed() || overlay.is_added() {
    *overlay = Projection::clone(styled);
  }
}

fn copy_hdr(is_hdr: bool, overlay_is_hdr: bool, mut overlay: EntityCommands) {
  match (is_hdr, overlay_is_hdr) {
    (true, false) => {
      overlay.insert(Hdr);
    }
    (false, true) => {
      overlay.remove::<Hdr>();
    }
    _ => {}
  }
}
