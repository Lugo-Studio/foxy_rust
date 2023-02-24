#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

use tracing::info;
use kemono::app::App;

mod components;

#[derive(Debug)]
struct Data {
  description: String,
  names: Vec<String>,
}

fn main() {
  let custom_data = Data {
    description: "Cute Girls".into(),
    names: vec![
      "Shirakami Fubuki".into(),
      "Jibril".into(),
      "Re=L Rayford".into(),
    ],
  };

  let mut app = App::new();
  app.insert_resource(custom_data);

  let d = app.get_resource::<Data>();
  info!("{d:#?}");

  app.run();
}