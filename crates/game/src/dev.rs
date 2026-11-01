use bevy::{
  dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
  prelude::*,
  text::FontSmoothing,
};

pub struct DevGamePlugin;

impl Plugin for DevGamePlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(FpsOverlayPlugin {
      config: FpsOverlayConfig {
        text_config: TextFont {
          font_size: FontSize::Px(32.0),
          font: default(),
          font_smoothing: FontSmoothing::default(),
          ..default()
        },
        text_color: Color::srgb(1.0, 0.0, 0.0),
        refresh_interval: core::time::Duration::from_millis(100),
        enabled: true,
        frame_time_graph_config: FrameTimeGraphConfig {
          enabled: true,
          min_fps: 30.0,
          target_fps: 144.0,
        },
      },
    });
  }
}
