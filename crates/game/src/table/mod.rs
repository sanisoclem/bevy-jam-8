use crate::ron_asset::ron_asset_plugin;
use bevy::{
  math::StableInterpolate,
  picking::{hover::Hovered, mesh_picking::MeshPickingPlugin},
  prelude::*,
  ui_render::UiMaterialPlugin,
};
use blueprint_file::{BLUEPRINT_PATH, GameBlueprintFile, GameBlueprintHandle, load_blueprint};
use card::{CardAtlas, CardHudMaterial, CardMaterial, CardPose};
use jam8::prelude::*;
use std::f32::consts::PI;
use sys_anim::AnimateTo;
use sys_cam::PlayerCamera;
use sys_style::Lens;

mod blueprint_file;
mod card;

pub use blueprint_file::{AlchemyPalette, TableScript};
pub use card::{Card, CardFace, CardMaker};

pub const CARD_ASPECT: f32 = 422.0 / 586.0;
pub const HUD_MARGIN_VH: f32 = 3.0;
pub const TARGET_HUD_VH: f32 = 22.0;

// TODO: move to game settings
const HAND_ROW: CardRow = CardRow {
  y: -0.37,
  size: 0.3,
  spacing: 0.22,
  lean: 0.25,
  fan: 0.06,
};
const OPPONENT_ROW: CardRow = CardRow {
  y: 0.42,
  size: 0.26,
  spacing: 0.22,
  lean: -0.15,
  fan: 0.0,
};

pub fn table_plugin(app: &mut App) {
  app
    .add_plugins((
      MeshPickingPlugin,
      MaterialPlugin::<CardMaterial>::default(),
      UiMaterialPlugin::<CardHudMaterial>::default(),
      ron_asset_plugin::<GameBlueprintFile>,
    ))
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      (load_blueprint.before(SimSystems), update_cards, update_hud),
    );
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CardSlot {
  pub hand: Entity,
  pub card: usize,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
#[require(AnimateTo)]
pub enum CardPlace {
  Row,
  Away,
  Pot,
  Showcase,
  Corner, // in HUD
}

#[derive(Component, Debug, Default)]
pub struct TableView {
  pub turn: usize,
  pub visible_turns: usize,
  pub frozen: bool,
}

#[derive(Component, Debug)]
pub struct TableHud;

#[derive(Component, Debug, Default)]
pub struct TargetHud {
  pub shown: bool,
}

#[derive(Component, Debug, Default)]
pub struct Overlay {
  pub alpha: f32,
}

struct CardRow {
  y: f32,
  size: f32,
  spacing: f32,
  lean: f32,
  fan: f32,
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
  commands.insert_resource(GameBlueprintHandle(asset_server.load(BLUEPRINT_PATH)));
  // hardcoded the card size in too many places...
  commands.insert_resource(CardAtlas {
    mesh: meshes.add(Rectangle::new(CARD_ASPECT, 1.0)),
    sheet: asset_server.load("cards/sheet.png"),
  });
}

// fn item_face(palette: &AlchemyPalette, item: AlchemyItem) -> CardFace {
//   CardFace {
//     shade: palette.shade(item.color),
//     icon: palette.icon(item.item_type),
//   }
// }

// fn display_cards(
//   mut dealt: MessageReader<RoundDealt>,
//   games: Query<(&Game, &Round)>,
//   player_hands: Query<&PlayerCardHand>,
//   opponent_hands: Query<&OpponentCardHand>,
//   palette: Res<AlchemyPalette>,
//   camera: Single<Entity, With<PlayerCamera>>,
//   shown: Query<Entity, Or<(With<Card>, With<TableHud>)>>,
//   mut fluid: ResMut<CauldronFluid>,
//   mut maker: CardMaker,
// ) {
//   let Some(RoundDealt { game }) = dealt.read().last() else {
//     return;
//   };
//   let Ok((
//     Game {
//       player, opponent, ..
//     },
//     round,
//   )) = games.get(*game)
//   else {
//     return;
//   };
//   let (Ok(player_hand), Ok(opponent_hand)) =
//     (player_hands.get(*player), opponent_hands.get(*opponent))
//   else {
//     return;
//   };
//   for entity in &shown {
//     maker.commands.entity(entity).despawn();
//   }
//   *fluid = CauldronFluid::default();
//
//   for (card, item) in player_hand.cards.iter().enumerate() {
//     maker
//       .spawn(item_face(&palette, *item))
//       .insert((
//         CardSlot {
//           hand: *player,
//           card,
//         },
//         ChildOf(*camera),
//       ))
//       .observe(on_card_clicked);
//   }
//   for (card, item) in opponent_hand.cards.iter().enumerate() {
//     if let Some(item) = item {
//       maker.spawn(item_face(&palette, *item)).insert((
//         CardSlot {
//           hand: *opponent,
//           card,
//         },
//         ChildOf(*camera),
//       ));
//     }
//   }
//
//   let target = maker.hud(item_face(&palette, round.target));
//   let label = |text: &str| {
//     (
//       Text::new(text),
//       TextFont {
//         font_size: FontSize::Vh(3.0),
//         ..default()
//       },
//       TextColor(Color::srgb(0.9, 0.9, 0.9)),
//     )
//   };
//   let button = |text: &str, action| {
//     (
//       Button,
//       Hovered::default(),
//       Node {
//         width: Val::Vh(20.0),
//         height: Val::Vh(6.0),
//         justify_content: JustifyContent::Center,
//         align_items: AlignItems::Center,
//         ..default()
//       },
//       BackgroundColor::default(),
//       action,
//       children![label(text)],
//     )
//   };
//   maker.commands.spawn((
//     TableHud,
//     Node {
//       width: percent(100),
//       height: percent(100),
//       ..default()
//     },
//     Pickable::IGNORE, // need to click through it
//     Visibility::Hidden,
//     children![
//       (
//         Node {
//           position_type: PositionType::Absolute,
//           top: Val::Vh(3.0),
//           right: Val::Vh(3.0),
//           width: Val::Vh(22.0 * 422.0 / 586.0),
//           height: Val::Vh(22.0),
//           ..default()
//         },
//         target,
//       ),
//       (
//         Node {
//           position_type: PositionType::Absolute,
//           top: Val::Vh(26.0),
//           right: Val::Vh(3.0),
//           width: Val::Vh(22.0 * 422.0 / 586.0),
//           justify_content: JustifyContent::Center,
//           ..default()
//         },
//         children![label("Target")],
//       ),
//       (
//         Node {
//           position_type: PositionType::Absolute,
//           right: Val::Vh(3.0),
//           bottom: Val::Vh(36.0),
//           flex_direction: FlexDirection::Column,
//           row_gap: Val::Vh(1.5),
//           ..default()
//         },
//         children![
//           button("Pass", TableButtonAction::Pass),
//           button("Reset", TableButtonAction::Reset),
//         ],
//       ),
//     ],
//   ));
// }

// fn update_cauldron(
//   mut played: MessageReader<CardPlayed>,
//   player_hands: Query<(), With<PlayerCardHand>>,
//   palette: Res<AlchemyPalette>,
//   mut cards: Query<(Entity, &mut CardSlot)>,
//   mut fluid: ResMut<CauldronFluid>,
//   mut commands: Commands,
// ) {
//   for CardPlayed {
//     hand,
//     card,
//     cauldron_item,
//     ..
//   } in played.read()
//   {
//     *fluid = CauldronFluid {
//       color: palette.shade(cauldron_item.color),
//       icon: Some(palette.icon(cauldron_item.item_type)),
//     };
//     let thrown = CardSlot {
//       hand: *hand,
//       card: *card,
//     };
//     for (entity, mut slot) in &mut cards {
//       if *slot == thrown {
//         commands.entity(entity).despawn();
//         slot.card = usize::MAX;
//       // just moving cards up
//       // TODO: helper to sync hand with display
//       } else if player_hands.contains(*hand) && slot.hand == *hand && slot.card > *card {
//         slot.card -= 1;
//       }
//     }
//   }
// }

fn update_cards(
  camera: Single<(&Projection, &Lens, &GlobalTransform), With<PlayerCamera>>,
  view: Single<(&TableView, &mut Transform)>,
  player_hands: Query<(), With<PlayerCardHand>>,
  time: Res<Time>,
  mut cards: Query<
    (
      &CardPlace,
      Option<&CardSlot>,
      &CardPose,
      Option<&Hovered>,
      &mut AnimateTo,
      &mut Transform,
      &mut Visibility,
    ),
    Without<TableView>,
  >,
) {
  let (projection, lens, camera_transform) = *camera;
  let Projection::Perspective(perspective) = projection else {
    return;
  };
  let (view, mut view_transform) = view.into_inner();
  let depth = 0.6;
  let screen_height = 2.0 * depth * (perspective.fov / 2.0).tan();
  view_transform.translation =
    Vec3::new(-lens.shake.x, lens.shake.y, 0.0) * screen_height - Vec3::Z * depth;
  let pot = camera_transform
    .affine()
    .inverse()
    .transform_point3(Vec3::new(0.0, 0.85, 0.0))
    - view_transform.translation;
  let corner = Vec3::new(
    perspective.aspect_ratio / 2.0 - (HUD_MARGIN_VH + TARGET_HUD_VH * CARD_ASPECT / 2.0) / 100.0,
    0.5 - (HUD_MARGIN_VH + TARGET_HUD_VH / 2.0) / 100.0,
    0.0,
  ) * screen_height;
  let mine = |slot: Option<&CardSlot>| slot.is_some_and(|x| player_hands.contains(x.hand));
  let hand_size = cards
    .iter()
    .filter(|(place, slot, ..)| **place == CardPlace::Row && mine(*slot))
    .count();

  let secs = time.elapsed_secs();
  for (place, slot, pose, hovered, mut animate, mut transform, mut visibility) in &mut cards {
    let row = if mine(slot) { &HAND_ROW } else { &OPPONENT_ROW };
    let (column, count) = match slot {
      Some(x) if mine(slot) => (x.card as f32, hand_size as f32),
      Some(x) => (x.card as f32 - view.turn as f32, view.visible_turns as f32),
      None => (0.0, 1.0),
    };
    let from_middle = column - (count - 1.0) / 2.0;
    let bob = 0.004 * (secs * 1.1 + pose.phase).sin();
    let arc = from_middle * from_middle * row.fan * 0.3;
    let fan = Quat::from_rotation_z(-from_middle * row.fan);
    let sway = Quat::from_rotation_z(0.025 * (secs * 0.8 + pose.phase * 1.7).sin());
    let in_row = Transform {
      translation: Vec3::new(from_middle * row.spacing, row.y - arc + bob, 0.0) * screen_height,
      rotation: fan * Quat::from_rotation_x(-row.lean) * pose.tilt * sway,
      scale: Vec3::splat(row.size * screen_height),
    };
    let off_edge = Vec3::Y * row.y.signum() * 0.5 * screen_height;
    let hovered = hovered.is_some_and(|x| x.get());

    let (to, half_life) = match place {
      CardPlace::Row if !mine(slot) && column >= count => {
        (in_row.with_translation(in_row.translation + off_edge), 0.07)
      }
      CardPlace::Row if !mine(slot) && view.frozen => (
        Transform {
          translation: in_row.translation + Vec3::Y * 0.06 * screen_height,
          rotation: in_row.rotation * Quat::from_rotation_x(-0.6),
          ..in_row
        },
        0.12,
      ),
      CardPlace::Row if hovered => (
        Transform {
          translation: in_row.translation + Vec3::new(0.0, 0.04, 0.12) * screen_height,
          rotation: fan * Quat::from_axis_angle(Vec3::new(1.0, 1.0, 0.0).normalize(), 0.35) * sway,
          ..in_row
        },
        0.06,
      ),
      CardPlace::Row => (in_row, 0.07),
      CardPlace::Away => (in_row.with_translation(in_row.translation + off_edge), 0.07),
      CardPlace::Pot => (
        Transform {
          translation: pot,
          rotation: Quat::from_rotation_x(-1.2),
          scale: Vec3::splat(row.size * 0.5 * screen_height),
        },
        0.09,
      ),
      CardPlace::Showcase => (
        Transform {
          translation: Vec3::Y * 0.06 * screen_height,
          rotation: Quat::from_rotation_y(0.3 * (secs * 1.3).sin())
            * Quat::from_rotation_x(0.12 * (secs * 0.9).sin()),
          scale: Vec3::splat(0.42 * screen_height),
        },
        0.15,
      ),
      // the turn on the way in and out is the spin
      CardPlace::Corner => (
        Transform {
          translation: corner,
          rotation: Quat::from_rotation_y(0.9 * PI),
          scale: Vec3::splat(0.001),
        },
        0.15,
      ),
    };
    *animate = AnimateTo {
      to,
      secs: half_life,
    };
    if *visibility == Visibility::Hidden {
      *transform = match place {
        CardPlace::Row => in_row.with_translation(in_row.translation + off_edge),
        _ => to,
      };
      *visibility = Visibility::Visible;
    }
  }
}

fn update_hud(
  time: Res<Time>,
  targets: Query<(&TargetHud, &mut UiTransform)>,
  veils: Query<(&Overlay, &mut BackgroundColor)>,
) {
  let delta = time.delta_secs();
  for (target, mut transform) in targets {
    let to = if target.shown { 1.0 } else { 0.0 };
    transform.scale.smooth_nudge(&Vec2::splat(to), 10.0, delta);
  }
  for (veil, mut background) in veils {
    let mut alpha = background.0.alpha();
    alpha.smooth_nudge(&veil.alpha, 4.0, delta);
    background.0.set_alpha(alpha);
  }
}
