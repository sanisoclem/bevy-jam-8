use bevy::prelude::*;
use sys_audio::SysAudioPlugin;
use sys_cam::SysCamPlugin;

pub struct Jam8Plugin;

impl Plugin for Jam8Plugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((SysAudioPlugin, SysCamPlugin));
  }
}
