use crate::objects::{self, SkySettings};
use bevy::prelude::*;
use sys_cam::{CameraPose, LookAroundCamera};

pub struct LookAroundGymPlugin;

impl Plugin for LookAroundGymPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (objects::spawner, setup).run_if(resource_added::<SkySettings>),
    );
  }
}

fn setup(mut commands: Commands) {
  commands.trigger(LookAroundCamera(CameraPose {
    eye: Vec3::new(0.0, 1.5, 3.7),
    target: Vec3::new(0.0, 0.8, 0.0),
  }));
}
