use std::sync::Arc;
use egui_winit_vulkano::Gui;
use tracing::{error, info, warn};
use tracing_unwrap::{OptionExt, ResultExt};
use vulkano::{
  swapchain::{
    AcquireError,
    PresentMode,
    Swapchain,
    SwapchainAcquireFuture,
    SwapchainCreateInfo,
    SwapchainCreationError,
    SwapchainPresentInfo,
    self
  },
  render_pass::{
    FramebufferCreateInfo,
    Framebuffer,
    RenderPass
  },
  image::{
    view::ImageView,
    SwapchainImage
  },
  command_buffer::{PrimaryAutoCommandBuffer},
  single_pass_renderpass,
  sync::{
    self,
    FlushError,
    GpuFuture
  }
};
use vulkano::format::Format;
use vulkano::swapchain::ColorSpace;
use crate::error::SwapchainError;
use crate::vulkan::device::EngineDevice;

pub struct EngineSwapchain {
  device: Arc<EngineDevice>,

  swapchain: Arc<Swapchain>,
  images: Vec<Arc<SwapchainImage>>,
  image_views: Vec<Arc<ImageView<SwapchainImage>>>,
  // depth_images: Vec<Arc<Image>>,

  framebuffers: Vec<Arc<Framebuffer>>,
  render_pass: Arc<RenderPass>,

  // image_available_semaphores: Vec<Semaphore>,
  // render_complete_semaphores: Vec<Semaphore>,
  // in_flight_fences: Vec<Fence>,
  // images_in_flight: Vec<Fence>,

  present_mode: PresentMode,

  previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl EngineSwapchain {
  const MAX_FRAMES_IN_FLIGHT: i32 = 2;

  pub fn new(device: Arc<EngineDevice>, requested_present_mode: PresentMode) -> Self {
    let (swapchain, images) = Self::create_vulkan_swapchain(device.clone(), requested_present_mode);
    let image_views = Self::create_image_views(&images);

    let render_pass = Self::create_render_pass(device.clone(), swapchain.clone());
    let framebuffers = Self::create_framebuffers(&image_views, render_pass.clone());

    let previous_frame_end = Some(Box::new(sync::now(device.device())) as Box<dyn GpuFuture>);

    Self {
      device,
      swapchain,
      images,
      image_views,
      // depth_images: vec![],
      framebuffers,
      render_pass,
      // image_available_semaphores: vec![],
      // render_complete_semaphores: vec![],
      // in_flight_fences: vec![],
      // images_in_flight: vec![],
      present_mode: requested_present_mode,
      previous_frame_end
    }
  }

  pub fn framebuffer(&self, index: u32) -> Arc<Framebuffer> {
    self.framebuffers[index as usize].clone()
  }

  pub fn image_view(&self, index: u32) -> Arc<ImageView<SwapchainImage>> {
    self.image_views[index as usize].clone()
  }

  pub fn render_pass(&self) -> Arc<RenderPass> {
    self.render_pass.clone()
  }

  pub fn present_mode(&self) -> PresentMode {
    self.present_mode
  }

  pub fn set_present_mode(&mut self, present_mode: PresentMode) -> Result<(), SwapchainError> {
    match Self::pick_present_mode(
      present_mode,
      self.device.swapchain_support().present_modes
    ) {
      Ok(mode) => {
        self.present_mode = mode;
        self.recreate();
        info!("Present mode: {:?}", mode);
        Ok(())
      }
      Err(err) => Err(err)
    }
  }

  pub fn acquire_next_image(&mut self) -> Option<(u32, SwapchainAcquireFuture)> {
    let (image_index, suboptimal, acquire_future) =
      match swapchain::acquire_next_image(self.swapchain.clone(), None) {
        Ok(result) => result,
        Err(AcquireError::OutOfDate) => {
          // info!("{:?}", self.device.viewport());
          self.recreate();
          return None;
        }
        Err(e) => {
          error!("Failed to acquire next swapchain image: {:?}", e);
          return None;
        }
      };

    if suboptimal {
      self.recreate();
    }

    Some((image_index, acquire_future))
  }

  pub fn present(
    &mut self,
    acquire_future: SwapchainAcquireFuture,
    buffer: PrimaryAutoCommandBuffer,
    gui: &mut Gui,
    image_index: u32
  ) {
    let after_future = gui.draw_on_image(acquire_future, self.image_view(image_index));

    let future = self.previous_frame_end
      .take()
      .unwrap_or_log()
      .join(after_future)
      .then_execute(self.device.queues().graphics, buffer)
      .unwrap_or_log()
      .then_swapchain_present(
        self.device.queues().graphics,
        SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index)
      )
      .then_signal_fence_and_flush();

    match future {
      Ok(future) => {
        self.previous_frame_end = Some(Box::new(future) as Box<_>);
      },
      Err(FlushError::OutOfDate) => {
        self.recreate();
        self.previous_frame_end = Some(Box::new(sync::now(self.device.device())) as Box<_>);
      },
      Err(e) => {
        error!("Failed to flush future: {:?}", e);
        self.previous_frame_end = Some(Box::new(sync::now(self.device.device())) as Box<_>);
      }
    }
  }

  pub fn end_previous_frame(&mut self) {
    self.previous_frame_end
        .as_mut()
        .take()
        .unwrap_or_log()
        .cleanup_finished();
  }

  fn create_vulkan_swapchain(device: Arc<EngineDevice>, requested_present_mode: PresentMode) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let support = device.swapchain_support();

    let (min_image_count, image_usage, composite_alpha) = (
      support.capabilities.min_image_count,
      support.capabilities.supported_usage_flags,
      support.capabilities.supported_composite_alpha.iter().next().unwrap_or_log()
    );

    let (image_format, image_color_space) = match Self::pick_format_and_colorspace(
      (Format::B8G8R8A8_SRGB, ColorSpace::SrgbNonLinear),
      support.formats
    ) {
      Ok((format, space)) => (Some(format), space),
      Err(err) => {
        warn!("{err}");
        (Some(Format::B8G8R8A8_UNORM), ColorSpace::SrgbNonLinear)
      }
    };

    let image_extent: [u32; 2] = device.window().inner_size().into();

    let present_mode = match Self::pick_present_mode(
      requested_present_mode,
      support.present_modes
    ) {
      Ok(mode) => mode,
      Err(err) => {
        warn!("{err}");
        PresentMode::Fifo
      }
    };

    let swapchain = Swapchain::new(
      device.device(),
      device.surface(),
      SwapchainCreateInfo {
        min_image_count,
        image_format,
        image_color_space,
        image_extent,
        image_usage,
        composite_alpha,
        present_mode,
        ..Default::default()
      }
    );

    match swapchain {
      Ok(result) => {
        info!("Present mode: {present_mode:?}");
        result
      },
      Err(e) => {
        error!("Failed to create swapchain: {e:?}");
        panic!();
      }
    }
  }

  pub fn recreate(&mut self) {
    let support = self.device.swapchain_support();
    let image_extent: [u32; 2] =
      self.device
        .window()
        .inner_size()
        .into();

    let present_mode = match Self::pick_present_mode(
      self.present_mode,
      support.present_modes
    ) {
      Ok(mode) => mode,
      Err(err) => {
        warn!("{err}");
        PresentMode::Fifo
      }
    };

    let (new_swapchain, new_images) = match self.swapchain.recreate(
      SwapchainCreateInfo {
        image_extent,
        present_mode,
        ..self.swapchain.create_info()
      }
    ) {
      Ok(result) => result,
      Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
        // warn!("Swapchain image extent not supported.");
        return;
      },
      Err(SwapchainCreationError::ImageExtentZeroLengthDimensions { .. }) => {
        // warn!("Swapchain image extent has zero dimension.");
        return;
      },
      Err(e) => {
        error!("Failed to recreate swapchain: {:?}", e);
        panic!();
      }
    };

    self.images = new_images;
    self.image_views = Self::create_image_views(&self.images);
    self.swapchain = new_swapchain;
    self.framebuffers = Self::create_framebuffers(
      &self.image_views,
      self.render_pass.clone()
    );
  }

  fn create_image_views(images: &[Arc<SwapchainImage>]) -> Vec<Arc<ImageView<SwapchainImage>>> {
    images.iter()
          .map(|image| ImageView::new_default(image.clone()).unwrap_or_log())
          .collect::<Vec<_>>()
  }

  fn create_render_pass(device: Arc<EngineDevice>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    single_pass_renderpass!(
      device.device(),
      attachments: {
        color: {
          load: Clear,
          store: Store,
          format: swapchain.image_format(),
          samples: 1,
        }
      },
      pass: {
        color: [color],
        depth_stencil: {}
      }
    ).expect_or_log("Failed to create Vulkan render pass")
  }

  fn create_framebuffers(
    image_views: &[Arc<ImageView<SwapchainImage>>],
    render_pass: Arc<RenderPass>,
  ) -> Vec<Arc<Framebuffer>> {
    image_views
      .iter()
      .map(|view| {
        Framebuffer::new(
          render_pass.clone(),
          FramebufferCreateInfo {
            attachments: vec![view.clone()],
            ..Default::default()
          },
        ).unwrap_or_log()
      })
      .collect::<Vec<_>>()
  }

  fn pick_present_mode(
    requested: PresentMode,
    supported: Vec<PresentMode>
  ) -> Result<PresentMode, SwapchainError> {
    if supported.iter().any(|m| *m == requested) {
      Ok(requested)
    } else {
      Err(SwapchainError::InvalidPresentMode(requested))
    }
  }

  fn pick_format_and_colorspace(
    requested: (Format, ColorSpace),
    supported: Vec<(Format, ColorSpace)>
  ) -> Result<(Format, ColorSpace), SwapchainError> {
    info!("{supported:?}");
    if supported.iter().any(|m| *m == requested) {
      Ok(requested)
    } else {
      Err(SwapchainError::InvalidFormatMode(requested))
    }
  }
}