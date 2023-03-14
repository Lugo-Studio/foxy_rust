#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

use kemono::app::{App};

fn main() {
  App::new(true)
    .run();
}