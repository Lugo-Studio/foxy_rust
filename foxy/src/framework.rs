use strum_macros::Display;
use winit::event_loop::{EventLoop, ControlFlow};

use crate::{
  graphics::Graphics,
  event::{WindowEvent, InputEvent}, 
  runnable::Runnable, prelude::Time,
};

pub struct Foxy {
  event_loop: EventLoop<()>,
  framework: Framework,
}

impl Foxy {
  pub fn builder() -> FrameworkBuilder {
    FrameworkBuilder::default()
  }

  pub fn run<A: 'static + Runnable>(self, foxy_app: A) {
    self.framework.run(self.event_loop, foxy_app);
  }
}

pub struct FrameworkBuilder {
  pub title: &'static str,
  pub width: u32,
  pub height: u32,
  pub centered: bool,
  pub tick_rate: f64,
  pub logging_level: Option<Level>,
  pub wgpu_logging_level: Option<Level>,
}

impl FrameworkBuilder {
  pub fn with_title(mut self, title: &'static str) -> Self {
    self.title = title;
    self
  }

  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.width = width;
    self.height = height;
    self
  }
  pub fn with_centered(mut self, centered: bool) -> Self {
    self.centered = centered;
    self
  }

  pub fn with_tick_rate(mut self, tick_rate: f64) -> Self {
    self.tick_rate = tick_rate;
    self
  }

  pub fn with_logging(mut self, level: Option<Level>) -> Self {
    self.logging_level = level;
    self
  }

  pub fn with_wgpu_logging(mut self, level: Option<Level>) -> Self {
    self.wgpu_logging_level = level;
    self
  }

  fn initialize_logger(&self) {
    // https://stackoverflow.com/questions/73247589/how-to-turn-off-tracing-events-emitted-by-other-crates?rq=1
    let filter = format!(
      "foxy={},wgpu={}",
      match &self.logging_level {
        None => "off".to_string(),
        Some(level) => level.to_string()
      },
      match &self.wgpu_logging_level {
        None => "off".to_string(),
        Some(level) => level.to_string()
      }
    );
    tracing_subscriber::fmt()
      .with_env_filter(filter)
      // .with_max_level(min_level)
      .with_thread_names(true)
      .init();
  }

  pub fn build<'a>(self) -> Foxy {
    self.initialize_logger();
    let time = Time::new(self.tick_rate, 1024);
    let (graphics, event_loop) = Graphics::new(self.title, self.width, self.height, self.centered);
    Foxy {
      event_loop,
      framework: Framework {
        time,
        graphics,
      },
    }
  }

  pub fn build_and_run<App: 'static + Runnable + Default>(self) {
    self.build().run(App::default());
  }
}

impl Default for FrameworkBuilder {
  fn default() -> Self {
    Self {
      title: "Foxy",
      width: 800,
      height: 500,
      centered: false,
      tick_rate: 128.,
      logging_level: None,
      wgpu_logging_level: None,
    }
  }
}

struct Framework{
  time: Time,
  graphics: Graphics,
}

impl Framework{
  fn run<A: 'static + Runnable>(mut self, event_loop: EventLoop<()>, mut app: A) {
    tracing::trace!("Entering Foxy Framework loop.");
    app.start(&mut self.graphics);
    event_loop.run(move |event, _, control_flow| {
      match event {
        winit::event::Event::WindowEvent { window_id: _, event } => {
          match event {
            winit::event::WindowEvent::CloseRequested => {
              *control_flow = ControlFlow::Exit;
            },
            winit::event::WindowEvent::Resized(_) => {
              app.window(&mut self.graphics, WindowEvent::Resized, &self.time);
            },
            winit::event::WindowEvent::Moved(_) => {
              app.window(&mut self.graphics, WindowEvent::Moved, &self.time);
            },
            winit::event::WindowEvent::KeyboardInput { device_id: _, input: _, is_synthetic: _ } => {
              app.input(&mut self.graphics, InputEvent::Keyboard, &self.time);
            },
            winit::event::WindowEvent::ModifiersChanged(_) => {
              app.input(&mut self.graphics, InputEvent::Modifiers, &self.time);
            },
            winit::event::WindowEvent::CursorMoved { device_id: _, position: _, .. } => {
              app.input(&mut self.graphics, InputEvent::Cursor, &self.time);
            },
            winit::event::WindowEvent::MouseWheel { device_id: _, delta: _, phase: _, .. } => {
              app.input(&mut self.graphics, InputEvent::Scroll, &self.time);
            },
            winit::event::WindowEvent::MouseInput { device_id: _, state: _, button: _, .. } => {
              app.input(&mut self.graphics, InputEvent::Mouse, &self.time);
            },
            _ => {},
          }
        },
        winit::event::Event::MainEventsCleared => {
          self.time.update();
          while self.time.should_do_tick() {
            self.time.tick();
            app.tick(&mut self.graphics, &self.time);
          }
          app.update(&mut self.graphics, &self.time);
          app.post_update(&mut self.graphics, &self.time);
          self.graphics.window().request_redraw();
        },
        winit::event::Event::RedrawRequested(_) => {
          self.graphics.render();
        },
        winit::event::Event::RedrawEventsCleared => {
          self.graphics.post_render();
        },
        winit::event::Event::LoopDestroyed => {
          app.stop(&mut self.graphics, &self.time);
          tracing::trace!("Exiting Foxy Framework loop.");
        },
        _ => {},
      }
    });
  }
}

#[derive(Default, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Level {
  Trace,
  Debug,
  #[default]
  Info,
  Warn,
  Error,
}