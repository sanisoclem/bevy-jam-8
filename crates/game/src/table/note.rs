use super::{
  AlchemyPalette, CardSlot, TABLE_DEPTH, TableView, TargetHud,
  card::{CardAtlas, paper},
};
use bevy::{
  light::{NotShadowCaster, NotShadowReceiver},
  pbr::{ExtendedMaterial, MaterialExtension},
  picking::hover::Hovered,
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};
use jam8::prelude::{
  AlchemyItemType, CauldronCard, Game, OpponentCardHand, PlayerCardHand, Round,
  get_relevant_recipes,
};
use sys_anim::AnimateTo;
use sys_cam::PlayerCamera;
use sys_style::Unstyled;

// NOTE: this is hard coded in the uniform too
const NOTE_LINES: usize = 16;

pub type NoteMaterial = ExtendedMaterial<StandardMaterial, NotePaint>;

#[derive(Component, Debug, Default)]
pub struct RecipeNote {
  lines: [Option<NoteLine>; NOTE_LINES],
  waiting: Vec<[AlchemyItemType; 3]>,
  length: f32,
  settled: bool,
}

#[derive(Clone, Copy, Debug)]
struct NoteLine {
  recipe: [AlchemyItemType; 3],
  written_secs: f32,
  erased_secs: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
enum NoteFocus {
  WithCauldronCard(AlchemyItemType, AlchemyItemType),
  WithoutCauldronCard(AlchemyItemType),
  Outcome(AlchemyItemType),
}

#[derive(ShaderType, Clone, Copy, PartialEq, Default, Debug)]
struct GpuNote {
  lines: [UVec4; NOTE_LINES],
  length: f32,
  current: u32,
  _webgl2_padding: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct NotePaint {
  #[uniform(100)]
  note: GpuNote,
  #[texture(101)]
  #[sampler(102)]
  sheet: Handle<Image>,
}

impl MaterialExtension for NotePaint {
  fn fragment_shader() -> ShaderRef {
    "shaders/note.wesl".into()
  }
}

pub fn on_note_added(
  added: On<Add<RecipeNote>>,
  atlas: Res<CardAtlas>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<NoteMaterial>>,
  mut commands: Commands,
) {
  let height = NOTE_LINES as f32 + 1.0;
  let mesh = Rectangle::new(4.8, height)
    .mesh()
    .build()
    .translated_by(Vec3::Y * -height / 2.0);
  commands.entity(added.entity).insert((
    Mesh3d(meshes.add(mesh)),
    MeshMaterial3d(materials.add(NoteMaterial {
      base: paper(),
      extension: NotePaint {
        note: GpuNote::default(),
        sheet: atlas.sheet.clone(),
      },
    })),
    AnimateTo::default(),
    NotShadowCaster,
    NotShadowReceiver,
    Pickable::IGNORE,
    Unstyled,
    Visibility::Hidden,
  ));
}

pub fn update_note(
  camera: Single<&Projection, With<PlayerCamera>>,
  view: Single<&TableView>,
  game: Single<(&Game, &Round, &CauldronCard)>,
  player_hands: Query<&PlayerCardHand>,
  opponent_hands: Query<&OpponentCardHand>,
  (cards, targets): (
    Query<(&CardSlot, &Hovered)>,
    Query<&Hovered, With<TargetHud>>,
  ),
  palette: Res<AlchemyPalette>,
  time: Res<Time>,
  notes: Query<(
    &mut RecipeNote,
    &MeshMaterial3d<NoteMaterial>,
    &mut AnimateTo,
    &mut Transform,
    &mut Visibility,
  )>,
  mut materials: ResMut<Assets<NoteMaterial>>,
) {
  let Projection::Perspective(perspective) = *camera else {
    return;
  };
  let (
    Game {
      player,
      opponent,
      blueprint,
    },
    round,
    cauldron,
  ) = *game;
  let (Ok(player_hand), Ok(opponent_hand)) =
    (player_hands.get(*player), opponent_hands.get(*opponent))
  else {
    return;
  };
  let rules = &blueprint.rules;
  let hovered = cards.iter().find(|(_, x)| x.get()).map(|(slot, _)| slot);
  let focus = if !view.settled {
    None
  } else if targets.iter().any(|x| x.get()) {
    Some(NoteFocus::Outcome(round.target.item_type))
  } else if let Some(slot) = hovered
    && slot.hand == *player
  {
    player_hand.cards.get(slot.card).map(|x| match cauldron.0 {
      Some(pot) => NoteFocus::WithCauldronCard(x.item_type, pot.item_type),
      None => NoteFocus::WithoutCauldronCard(x.item_type),
    })
  } else {
    hovered
      .and_then(|x| opponent_hand.cards.get(x.card).copied().flatten())
      .map(|x| NoteFocus::WithoutCauldronCard(x.item_type))
  };

  let screen_height = 2.0 * TABLE_DEPTH * (perspective.fov / 2.0).tan();
  let secs = time.elapsed_secs();
  let delta = time.delta_secs();
  for (note, material, mut animate, mut transform, mut visibility) in notes {
    let note = note.into_inner();
    if view.settled && !note.settled {
      // get all types at play
      let shown = opponent_hand
        .cards
        .iter()
        .skip(round.turn)
        .take(view.visible_turns)
        .flatten();
      let mut types = player_hand
        .cards
        .iter()
        .chain(shown)
        .chain(&cauldron.0)
        .map(|x| x.item_type)
        .collect::<Vec<_>>();
      types.sort();
      types.dedup();
      let mut recipes = get_relevant_recipes(rules, round, cauldron, player_hand, opponent_hand);

      // TODO: include in solver
      for (x, first) in types.iter().enumerate() {
        for second in &types[x + 1..] {
          let recipe = [*first, *second, rules.combine_types(*first, *second)];
          if !recipes.contains(&recipe) {
            recipes.push(recipe);
          }
        }
      }
      for line in note.lines.iter_mut().flatten() {
        if line.erased_secs.is_none() && !recipes.contains(&line.recipe) {
          line.erased_secs = Some(secs);
        }
      }
      note.waiting = recipes
        .into_iter()
        .filter(|x| {
          !note
            .lines
            .iter()
            .flatten()
            .any(|y| y.erased_secs.is_none() && y.recipe == *x)
        })
        .collect();
    }
    note.settled = view.settled;

    for slot in &mut note.lines {
      if slot.is_some_and(|x| x.erased_secs.is_some_and(|y| secs - y > 0.3)) {
        *slot = None;
      }
    }

    // only add new lines once we have cleared the ones not in use
    if note.lines.iter().flatten().all(|x| x.erased_secs.is_none()) {
      for recipe in note.waiting.drain(..) {
        if let Some(slot) = note.lines.iter_mut().find(|x| x.is_none()) {
          *slot = Some(NoteLine {
            recipe,
            written_secs: secs,
            erased_secs: None,
          });
        }
      }
    }

    let length = match note.lines.iter().rposition(Option::is_some) {
      Some(x) => x as f32 + 2.0,
      None if note.length > 0.0 => 1.0,
      None => 0.0,
    };
    note.length += (length - note.length).clamp(-delta * 3.0, delta * 3.0);

    let trans_visibile = Transform {
      translation: Vec3::new(
        0.15 - perspective.aspect_ratio / 2.0,
        0.3 + 0.004 * (secs * 0.9).sin(),
        0.0,
      ) * screen_height,
      rotation: Quat::from_rotation_y(0.3)
        * Quat::from_rotation_x(-0.08)
        * Quat::from_rotation_z(0.02 * (secs * 0.7).sin()),
      scale: Vec3::splat(0.045 * screen_height),
    };
    let trans_hidden =
      trans_visibile.with_translation(trans_visibile.translation - Vec3::X * 0.35 * screen_height);
    *animate = AnimateTo {
      to: if note.length > 0.0 {
        trans_visibile
      } else {
        trans_hidden
      },
      secs: 0.15,
    };

    if *visibility == Visibility::Hidden {
      *transform = trans_hidden;
      *visibility = Visibility::Visible;
    }

    let mut gpu = GpuNote {
      length: note.length,
      current: match focus {
        Some(
          NoteFocus::WithCauldronCard(x, _)
          | NoteFocus::WithoutCauldronCard(x)
          | NoteFocus::Outcome(x),
        ) => palette.icon(x),
        None => 0,
      },
      ..default()
    };
    for (i, line) in note.lines.iter().enumerate() {
      let Some(line) = line else {
        continue;
      };
      let [input1, input2, result] = line.recipe;
      let focused = match focus {
        Some(NoteFocus::WithCauldronCard(x, pot)) => [input1, input2] == [x.min(pot), x.max(pot)],
        Some(NoteFocus::WithoutCauldronCard(x)) => [input1, input2].contains(&x),
        Some(NoteFocus::Outcome(x)) => result == x,
        None => false,
      };
      let icons = line.recipe.map(|x| palette.icon(x));
      let age = secs - line.erased_secs.unwrap_or(line.written_secs);
      gpu.lines[i] = UVec4::new(
        icons[0] | icons[1] << 8 | icons[2] << 16,
        ((age * 1000.0) as u32).min(5000),
        1 | u32::from(line.erased_secs.is_some()) << 1 | u32::from(focused) << 2,
        0,
      );
    }
    if let Some(mut paint) = materials.get_mut(&material.0)
      && paint.extension.note != gpu
    {
      paint.extension.note = gpu;
    }
  }
}
