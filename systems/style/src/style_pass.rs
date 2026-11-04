use bevy::{
  core_pipeline::{Core3dSystems, tonemapping::tonemapping},
  ecs::{schedule::ScheduleConfigs, system::BoxedSystem},
  prelude::*,
};

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
struct StylePass;

pub fn schedule_in_style_pass(
  system: ScheduleConfigs<BoxedSystem>,
) -> ScheduleConfigs<BoxedSystem> {
  schedule_after_tonemapping(system).in_set(StylePass)
}

pub fn schedule_after_style_pass(
  system: ScheduleConfigs<BoxedSystem>,
) -> ScheduleConfigs<BoxedSystem> {
  schedule_after_tonemapping(system).after(StylePass)
}

fn schedule_after_tonemapping(
  system: ScheduleConfigs<BoxedSystem>,
) -> ScheduleConfigs<BoxedSystem> {
  system.in_set(Core3dSystems::PostProcess).after(tonemapping)
}
