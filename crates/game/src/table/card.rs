use bevy::{
  ecs::system::{EntityCommands, SystemParam},
  light::{NotShadowCaster, NotShadowReceiver},
  pbr::{ExtendedMaterial, MaterialExtension},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
  ui_render::ui_material::{MaterialNode, UiMaterial},
};
use std::f32::consts::TAU;

const FRAME_VARIANTS: u32 = 4;

pub type CardMaterial = ExtendedMaterial<StandardMaterial, CardPaint>;

#[derive(Component, Debug)]
pub struct Card;

#[derive(Component, Debug)]
pub struct CardPose {
  pub tilt: Quat,
  pub phase: f32,
}

#[derive(Resource)]
pub struct CardAtlas {
  pub mesh: Handle<Mesh>,
  pub sheet: Handle<Image>,
}

#[derive(Clone, Copy, Debug)]
pub struct CardFace {
  pub shade: Color,
  pub icon: u32,
}

impl CardFace {
  fn uniform(self) -> GpuCardFace {
    GpuCardFace {
      panel: self.shade.into(),
      icon: self.icon as f32,
      outer_frame: fastrand::u32(0..FRAME_VARIANTS) as f32,
      inner_frame: fastrand::u32(0..FRAME_VARIANTS) as f32,
      _webgl2_padding: 0.0,
    }
  }
}

#[derive(ShaderType, Clone, Copy, Debug)]
struct GpuCardFace {
  panel: LinearRgba,
  icon: f32,
  outer_frame: f32,
  inner_frame: f32,
  _webgl2_padding: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct CardPaint {
  #[uniform(100)]
  face: GpuCardFace,
  #[texture(101)]
  #[sampler(102)]
  sheet: Handle<Image>,
}

impl MaterialExtension for CardPaint {
  fn fragment_shader() -> ShaderRef {
    "shaders/card.wesl".into()
  }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct CardHudMaterial {
  #[uniform(0)]
  face: GpuCardFace,
  #[texture(1)]
  #[sampler(2)]
  sheet: Handle<Image>,
}

impl UiMaterial for CardHudMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/card_hud.wesl".into()
  }
}

#[derive(SystemParam)]
pub struct CardMaker<'w, 's> {
  pub commands: Commands<'w, 's>,
  atlas: Res<'w, CardAtlas>,
  materials: ResMut<'w, Assets<CardMaterial>>,
  hud_materials: ResMut<'w, Assets<CardHudMaterial>>,
}

impl CardMaker<'_, '_> {
  pub fn spawn(&mut self, face: CardFace) -> EntityCommands<'_> {
    let material = self.materials.add(CardMaterial {
      base: StandardMaterial {
        perceptual_roughness: 0.92,
        reflectance: 0.25,
        diffuse_transmission: 0.35,
        double_sided: true,
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        ..default()
      },
      extension: CardPaint {
        face: face.uniform(),
        sheet: self.atlas.sheet.clone(),
      },
    });
    let jitter = || (fastrand::f32() - 0.5) * 0.08;
    self.commands.spawn((
      Card,
      CardPose {
        tilt: Quat::from_euler(EulerRot::XYZ, jitter(), jitter(), jitter()),
        phase: fastrand::f32() * TAU,
      },
      Mesh3d(self.atlas.mesh.clone()),
      MeshMaterial3d(material),
      NotShadowCaster,
      NotShadowReceiver,
      Visibility::Hidden,
    ))
  }

  pub fn hud(&mut self, face: CardFace) -> MaterialNode<CardHudMaterial> {
    MaterialNode(self.hud_materials.add(CardHudMaterial {
      face: face.uniform(),
      sheet: self.atlas.sheet.clone(),
    }))
  }
}
