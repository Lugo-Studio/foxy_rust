#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowEvent {
  Moved,
  Resized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
  Mouse,
  Keyboard,
  Modifiers,
  Cursor,
  Scroll,
}