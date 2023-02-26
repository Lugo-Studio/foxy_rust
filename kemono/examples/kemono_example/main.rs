#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

use kemono::app::AppBuilder;
use kemono::lifecycle::Event;
use legion::system;
use tracing::info;

mod components;

#[derive(Debug)]
struct Names {
  description: String,
  names: Vec<String>,
}

#[derive(Debug)]
struct Counter(i32);

fn main() {
  let custom_data = Names {
    description: "Cute Girls".into(),
    names: vec![
      "Shirakami Fubuki".into(),
      "Jibril".into(),
      "Stella Vermillion".into(),
      "Re=L Rayford".into(),
    ],
  };

  AppBuilder::new()
    .title("Example App")
    .tick_rate(1.)
    .insert_resource(custom_data)
    .insert_resource(Counter(1))
    .insert_system(Event::Start, print_names_system())
    .insert_system(Event::Tick, test_system())
    .build_and_run();
}

#[system(simple)]
fn print_names(#[resource] data: &Names) {
  info!("{:?}", data);
}

#[system(simple)]
fn test(#[resource] counter: &mut Counter) {
  if counter.0 % 3 == 0 {
    info!("{} Fox, Fubuki", counter.0);
  } else if counter.0 % 10 == 0 {
    info!("{} Fox, Kawaii-ooo", counter.0);
    counter.0 = 0;
  } else {
    info!("{} Fox", counter.0);
  }
  counter.0 += 1;
}