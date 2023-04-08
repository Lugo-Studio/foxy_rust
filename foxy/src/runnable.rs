use crate::{graphics::Graphics, event::{InputEvent, WindowEvent}, prelude::Time};

#[allow(unused)]
pub trait Runnable {
  fn start(&mut self, gfx: &mut Graphics) {}
  fn update(&mut self, gfx: &mut Graphics, time: &Time) {}
  fn post_update(&mut self, gfx: &mut Graphics, time: &Time) {}
  fn tick(&mut self, gfx: &mut Graphics, time: &Time) {}
  fn stop(&mut self, gfx: &mut Graphics, time: &Time) {}
  fn window(&mut self, gfx: &mut Graphics, event: WindowEvent, time: &Time) {}
  fn input(&mut self, gfx: &mut Graphics, event: InputEvent, time: &Time) {}
}