use bevy::{
  camera::RenderTarget,
  color::Luminance,
  core_pipeline::tonemapping::Tonemapping,
  ecs::system::{EntityCommands, SystemParam},
  math::Affine2,
  prelude::*,
  render::render_resource::TextureFormat,
  ui::UiTargetCamera,
};

const COLUMNS: u32 = 4;
const ROWS: u32 = 6;
const CELL: UVec2 = UVec2::new(192, 256);
const ATLAS: UVec2 = UVec2::new(CELL.x * COLUMNS, CELL.y * ROWS);

#[derive(Component, Debug)]
pub struct Card {
  cell: usize,
}

#[derive(Resource)]
pub struct CardAtlas {
  mesh: Handle<Mesh>,
  cells: Vec<AtlasCell>,
}

struct AtlasCell {
  root: Entity,
  label: Entity,
  material: Handle<StandardMaterial>,
  used: bool,
}

pub struct CardFace {
  text: String,
  shade: Color,
}

impl CardFace {
  pub fn new(text: impl Into<String>, shade: Color) -> Self {
    Self {
      text: text.into(),
      shade,
    }
  }
}

impl CardAtlas {
  pub fn new(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
  ) -> Self {
    // HACK: for now, generate textures by rendering text to a camera
    // TODO: replacing it with my magnificent infitity paint art
    let texture = Image::new_target_texture(ATLAS.x, ATLAS.y, TextureFormat::Rgba8UnormSrgb, None);
    let image = images.add(texture);
    let camera = commands
      .spawn((
        Camera2d,
        Camera {
          order: -1,
          ..default()
        },
        RenderTarget::from(image.clone()),
        Tonemapping::None,
        Msaa::Off,
      ))
      .id();
    let board = commands
      .spawn((
        Node {
          width: px(ATLAS.x as f32),
          height: px(ATLAS.y as f32),
          ..default()
        },
        UiTargetCamera(camera),
      ))
      .id();
    let cells = (0..COLUMNS * ROWS)
      .map(|x| {
        let corner = UVec2::new(x % COLUMNS, x / COLUMNS) * CELL;
        let root = commands
          .spawn((
            Node {
              position_type: PositionType::Absolute,
              left: px(corner.x as f32),
              top: px(corner.y as f32),
              width: px(CELL.x as f32),
              height: px(CELL.y as f32),
              justify_content: JustifyContent::Center,
              align_items: AlignItems::Center,
              ..default()
            },
            ChildOf(board),
          ))
          .id();
        let label = commands
          .spawn((
            Text::default(),
            TextFont {
              font_size: FontSize::Px(34.0),
              ..default()
            },
            TextLayout {
              justify: Justify::Center,
              ..default()
            },
            ChildOf(root),
          ))
          .id();
        let bleed_margin = Vec2::ONE;
        let material = materials.add(StandardMaterial {
          base_color_texture: Some(image.clone()),
          uv_transform: Affine2::from_scale_angle_translation(
            (CELL.as_vec2() - 2.0 * bleed_margin) / ATLAS.as_vec2(),
            0.0,
            (corner.as_vec2() + bleed_margin) / ATLAS.as_vec2(),
          ),
          unlit: true,
          ..default()
        });
        AtlasCell {
          root,
          label,
          material,
          used: false,
        }
      })
      .collect();
    Self {
      mesh: meshes.add(Rectangle::new(0.75, 1.0)),
      cells,
    }
  }
}

#[derive(SystemParam)]
pub struct CardMaker<'w, 's> {
  pub commands: Commands<'w, 's>,
  atlas: ResMut<'w, CardAtlas>,
}

impl CardMaker<'_, '_> {
  pub fn spawn(&mut self, face: CardFace, place: Transform) -> EntityCommands<'_> {
    let cell = self
      .atlas
      .cells
      .iter()
      .position(|x| !x.used)
      .expect("no space");
    self.atlas.cells[cell].used = true;
    let card = Card { cell };
    self.relabel(&card, face);
    let mesh = self.atlas.mesh.clone();
    let material = self.atlas.cells[cell].material.clone();
    self
      .commands
      .spawn((card, Mesh3d(mesh), MeshMaterial3d(material), place))
  }

  pub fn relabel(&mut self, card: &Card, face: CardFace) {
    let (root, label) = (
      self.atlas.cells[card.cell].root,
      self.atlas.cells[card.cell].label,
    );
    let ink = if face.shade.luminance() > 0.45 {
      Color::BLACK
    } else {
      Color::WHITE
    };
    self
      .commands
      .entity(root)
      .insert(BackgroundColor(face.shade));
    self
      .commands
      .entity(label)
      .insert((Text::new(face.text), TextColor(ink)));
  }

  pub fn despawn(&mut self, entity: Entity, card: &Card) {
    self.commands.entity(entity).despawn();
    self.atlas.cells[card.cell].used = false;
    self.relabel(card, CardFace::new("", Color::NONE));
  }
}
