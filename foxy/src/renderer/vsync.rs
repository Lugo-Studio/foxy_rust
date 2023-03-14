use vulkano::swapchain::PresentMode;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VsyncMode {
  Enabled,
  Disabled,
  Hybrid,
}

impl VsyncMode {
  pub fn to_present_mode(&self) -> PresentMode {
    match self {
      VsyncMode::Enabled => PresentMode::Fifo,
      VsyncMode::Disabled => PresentMode::Immediate,
      VsyncMode::Hybrid => PresentMode::Mailbox,
    }
  }
}

impl From<PresentMode> for VsyncMode {
  fn from(value: PresentMode) -> Self {
    match value {
      PresentMode::Fifo      => VsyncMode::Enabled ,
      PresentMode::Immediate => VsyncMode::Disabled,
      PresentMode::Mailbox   => VsyncMode::Hybrid  ,
      _                      => VsyncMode::Hybrid
    }
  }
}