use bevy::{picking::events::PointerClick, prelude::*};

pub fn caption_plugin(app: &mut App) {
  app
    .add_systems(Update, type_text)
    .add_observer(on_dialog_added);
}

#[derive(Component, Debug)]
pub struct Dialogue {
  text: String,
  shown: f32,
}

impl Dialogue {
  pub fn new(text: impl Into<String>) -> Self {
    Self {
      text: text.into(),
      shown: 0.0,
    }
  }

  fn finished(&self) -> bool {
    self.shown >= self.text.chars().count() as f32
  }
}

fn on_dialog_added(added: On<Add<Dialogue>>, mut commands: Commands) {
  let font = TextFont {
    font_size: FontSize::Vh(3.2),
    ..default()
  };
  commands
    .entity(added.entity)
    .insert((
      Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexEnd,
        align_items: AlignItems::Center,
        padding: UiRect::bottom(Val::Vh(9.0)),
        ..default()
      },
      GlobalZIndex(2),
      children![(
        Node {
          max_width: Val::Vh(90.0),
          padding: UiRect::axes(Val::Vh(3.0), Val::Vh(2.0)),
          //border_radius: BorderRadius::all(Val::Vh(1.5)),
          ..default()
        },
        Text::default(),
        font.clone(),
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextLayout::justify(Justify::Center),
        BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.75)),
        Pickable::IGNORE,
        children![(TextSpan::default(), font, TextColor(Color::NONE))],
      )],
    ))
    .observe(on_txt_clicked);
}

fn type_text(
  keys: Res<ButtonInput<KeyCode>>,
  time: Res<Time>,
  txts: Query<(Entity, &mut Dialogue, &Children)>,
  mut writer: TextUiWriter,
  mut commands: Commands,
) {
  for (entity, mut caption, children) in txts {
    if keys.just_pressed(KeyCode::Enter) {
      if caption.finished() {
        commands.entity(entity).despawn();
        continue;
      }
      caption.shown = f32::INFINITY;
    }
    caption.shown += time.delta_secs() * 45.0;
    let split = caption
      .text
      .char_indices()
      .nth(caption.shown as usize)
      .map_or(caption.text.len(), |(x, _)| x);
    let (shown, hidden) = caption.text.split_at(split);

    // TODO: add a mumbler effect and natural cadence (this is probably next jam territory)
    writer.text(children[0], 0).set_if_neq(shown.into());
    writer.text(children[0], 1).set_if_neq(hidden.into());
  }
}

fn on_txt_clicked(click: On<PointerClick>, mut txts: Query<&mut Dialogue>, mut commands: Commands) {
  let Ok(mut caption) = txts.get_mut(click.entity) else {
    return;
  };
  if caption.finished() {
    // TODO: animate out
    commands.entity(click.entity).despawn();
  } else {
    caption.shown = f32::INFINITY;
  }
}
