use std::ffi::CString;
use std::os::raw::c_char;
use winit::window::Window;
use ash::extensions::khr::Swapchain;
use ash::prelude::VkResult;
use ash::vk::{ Buffer, ClearColorValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, DescriptorSetLayoutBinding, DeviceMemory, DeviceSize, Extent2D, Fence, FenceCreateFlags, FenceCreateInfo, Framebuffer, FramebufferCreateInfo, Offset2D, PhysicalDeviceMemoryProperties, PipelineBindPoint, PipelineLayout, PresentModeKHR, Rect2D, RenderPass, RenderPassBeginInfo, Semaphore, SemaphoreCreateInfo, SubpassContents, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SwapchainKHR };
use ash::{ vk, vk::QueueFlags, vk::SurfaceKHR,Entry, extensions::khr::Surface };
use raw_window_handle::{ HasRawWindowHandle, HasRawDisplayHandle };

use super::pipeline::PipelineConfig;
use super::vulkan_resources::{Vertex, VulkanResources};

pub struct VulkanInstance {
  _entry: Entry,
  instance                        : ash::Instance,
  physical_device                 : Option<vk::PhysicalDevice>,
  logical_device                  : Option<ash::Device>,
  surface                         : Option<SurfaceKHR>,
  surface_format                  : Option<SurfaceFormatKHR>,
  surface_capabilities            : Option<SurfaceCapabilitiesKHR>,
  presentation_mode               : Option<PresentModeKHR>,
  swap_extent                     : Option<Extent2D>,
  swapchain_image_format          : Option<vk::Format>,
  surface_loader                  : Option<Surface>,
  graphics_queue_family_index     : Option<u32>,
  presentation_queue_family_index : Option<u32>,
  graphics_queue                  : Option<vk::Queue>,
  swapchain_loader                : Option<Swapchain>,
  swapchain                       : Option<SwapchainKHR>,
  swapchain_images                : Option<Vec<vk::Image>>,
  swapchain_framebuffers          : Option<Vec<Framebuffer>>,
  swapchain_image_views           : Option<Vec<vk::ImageView>>,
  render_pass                     : Option<RenderPass>,
  vulkan_resources                : Option<VulkanResources>,
  vertex_count                    : usize,                // Need to move to vulkan_resources
  vertex_buffer                   : Option<Buffer>,       // Need to move to vulkan_resources
  vertex_buffer_memory            : Option<DeviceMemory>, // Need to move to vulkan_resources
  command_pool                    : Option<CommandPool>,
  command_buffers                 : Option<Vec<CommandBuffer>>,
  image_available_semaphores      : Vec<Semaphore>,
  render_complete_semaphores      : Vec<Semaphore>,
  in_flight_fences                : Vec<Fence>,
  images_in_flight                : Vec<Option<Fence>>,
}

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

impl VulkanInstance {
  
  pub fn new(app_name: &str, engine_name: &str) -> Result<Self, vk::Result> {

    let entry = unsafe { match Entry::load() {
      Ok(entry) => entry,
      Err(_)    => return Err(vk::Result::ERROR_INITIALIZATION_FAILED),
    } };

    println!("\nInitializing Vulkan Instance");

    let app_name_cstr = CString::new(app_name).unwrap();
    let engine_name_cstr = CString::new(engine_name).unwrap();
    
    let app_info = vk::ApplicationInfo::builder()
      .application_name(&app_name_cstr)
      .application_version(vk::make_api_version(0, 0, 0, 0))
      .engine_name(&engine_name_cstr)
      .engine_version(vk::make_api_version(0, 0, 0, 0))
      .api_version(vk::API_VERSION_1_0);

    let instance_extensions = VulkanInstance::load_instance_extensions();
    let create_info = vk::InstanceCreateInfo::builder()
      .application_info(&app_info)
      .enabled_extension_names(&instance_extensions);

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    Ok(VulkanInstance {
      _entry: entry, 
      instance,
      physical_device                 : None,
      logical_device                  : None,
      surface                         : None,
      surface_capabilities            : None,
      surface_format                  : None,
      presentation_mode               : None,
      swap_extent                     : None,
      swapchain_image_format          : None,
      surface_loader                  : None,
      graphics_queue_family_index     : None,
      presentation_queue_family_index : None,
      graphics_queue                  : None,
      swapchain_loader                : None,
      swapchain                       : None,
      swapchain_images                : None,
      swapchain_framebuffers          : None,
      swapchain_image_views           : None,
      render_pass                     : None,
      vulkan_resources                : None,
      vertex_buffer                   : None,
      vertex_buffer_memory            : None,
      command_pool                    : None,
      command_buffers                 : None,
      image_available_semaphores      : Vec::new(),
      render_complete_semaphores      : Vec::new(),
      vertex_count                    : 0,
      in_flight_fences                : Vec::new(),
      images_in_flight                : Vec::new(), 
    })
  }

  pub unsafe fn create_surface(&mut self, window: &Window) -> Result<&mut Self, vk::Result> {

    if (self.surface_loader.is_none()) {
      self.surface_loader = Some(Surface::new(&self._entry, &self.instance));
    }

    let raw_window_handle   = window.raw_window_handle();
    let raw_display_handle = window.raw_display_handle();

    let surface = ash_window::create_surface(
      &self._entry,
      &self.instance,
      raw_display_handle,
      raw_window_handle,
      None
    )?;
    
    self.surface = Some(surface);
    Ok(self)
  }

  pub fn configure_hardware(&mut self) -> &mut Self {
  
    let device = self.select_physical_device();
    self.physical_device = match device {
      Ok(device) => Some(device),
      Err(_) => {
        panic!("No compatible device found");
      }
    };

    let physical_device = match self.physical_device {
      Some(device) => device,
      None => {
        panic!("Error referencing VulkanInstance field: physical_device");
      }
    };

    let properties = unsafe { self.instance.get_physical_device_properties(physical_device) };
    let features     = unsafe { self.instance.get_physical_device_features(physical_device) };

    println!("\nDevice Properties -\n{}", VulkanInstance::format_device_properties(properties));
    println!("\nDevice Features -\n{}", VulkanInstance::format_device_features(features));

    self.surface_loader = Some(Surface::new(&self._entry, &self.instance));
    let queue_indicies = self.identify_required_queue_family_indices(physical_device, &self.instance);
    match queue_indicies {
      Some((graphics_queue_index, presentation_queue_index)) => {

        self.graphics_queue_family_index     = Some(graphics_queue_index);
        self.presentation_queue_family_index = Some(presentation_queue_index);

        println!("Configured Queue indices (Graphics: {}, Presentation: {})", graphics_queue_index, presentation_queue_index);
        self
      },
      None => {panic!("Failed to configure required Vulkan Queues, no indices detected.")}
    }
  }

  pub fn create_logical_device(&mut self) -> Result<&mut Self, vk::Result> {
    let queue_priorities = [1.0_f32];
    let queue_family_index = self.graphics_queue_family_index.unwrap();

    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
      .queue_family_index(queue_family_index)
      .queue_priorities(&queue_priorities)
      .build();

    let device_extension_names = [
      ash::extensions::khr::Swapchain::name().as_ptr(),
    ];

    let physical_device_features = vk::PhysicalDeviceFeatures::default();

    let device_create_info = vk::DeviceCreateInfo::builder()
      .queue_create_infos(&[queue_create_info])
      .enabled_features(&physical_device_features)
      .enabled_extension_names(&device_extension_names)
      .build();

    let logical_device = unsafe {
      self.instance.create_device(self.physical_device.unwrap(), &device_create_info, None)?
    };

    self.logical_device = Some(logical_device);

    let graphics_queue = unsafe {
      self.logical_device.as_ref().unwrap().get_device_queue(queue_family_index, 0)
    };

    self.graphics_queue = Some(graphics_queue);

    Ok(self)
  }

  pub fn create_swapchain(&mut self, window: &Window) -> Result<&mut Self, vk::Result> {

    let _ = self.query_surface_capabilities().unwrap()
      .configure_surface_format().unwrap()
      .configure_presentation_mode().unwrap()
      .configure_swap_extent(window);

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
      .surface(self.surface.unwrap())
      .min_image_count(self.surface_capabilities.as_ref().unwrap().min_image_count + 1)
      .image_format(self.surface_format.as_ref().unwrap().format)
      .image_color_space(self.surface_format.as_ref().unwrap().color_space)
      .image_extent(self.swap_extent.unwrap())
      .image_array_layers(1)
      .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
      .pre_transform(self.surface_capabilities.as_ref().unwrap().current_transform)
      .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
      .present_mode(self.presentation_mode.unwrap())
      .clipped(true);

    let swapchain_loader = Swapchain::new(&self.instance, self.logical_device.as_ref().unwrap());
    let swapchain = unsafe { 
      swapchain_loader.create_swapchain(&swapchain_create_info, None)?
    };

    self.swapchain = Some(swapchain);
    self.swapchain_loader = Some(swapchain_loader);

    let swapchain_images = self.get_swapchain_images();
    if let Ok(images) = swapchain_images {
      self.swapchain_images = Some(images);
    } else {
      return Err(swapchain_images.err().unwrap())
    }

    if let Some(ref images) = self.swapchain_images {
      let swapchain_image_views = self.create_image_views(&images[..]);
      match swapchain_image_views {
        Ok(views) => self.swapchain_image_views = Some(views),
        Err(e) => return Err(e),
      }
    } else {
      panic!("Swapchain Image View creation failed");
    }

    Ok(self)
  }

  fn get_swapchain_images(&self) -> VkResult<Vec<vk::Image>> {
    let swapchain_images = unsafe {
      self.swapchain_loader.as_ref().unwrap().get_swapchain_images(self.swapchain.unwrap())?
    };

    Ok(swapchain_images)
  }

  fn create_image_views(&self, swapchain_images: &[vk::Image]) -> VkResult<Vec<vk::ImageView>> {
    let views: Vec<vk::ImageView> = swapchain_images.iter().map(|&image| {
      let create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(self.swapchain_image_format.unwrap())
        .components(vk::ComponentMapping::default())
        .subresource_range(vk::ImageSubresourceRange {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          base_mip_level: 0,
          level_count: 1,
          base_array_layer: 0,
          layer_count: 1,
        })
        .build();

      unsafe {
        self.logical_device.as_ref().unwrap().create_image_view(&create_info, None)
      }
    }).collect::<Result<Vec<_>, _>>()?;
    Ok(views)
  }

  pub fn create_framebuffers(&mut self) -> &mut Self {

    self.swapchain_framebuffers = Some(self.swapchain_image_views.iter().filter_map(|image_view| {

        //let attachments = [image_view];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(self.render_pass.unwrap()) 
            .attachments(&image_view)
            .width(self.swap_extent.unwrap().width)
            .height(self.swap_extent.unwrap().height)
            .layers(1)
            .build();

        unsafe {
            self.logical_device.as_ref().unwrap().create_framebuffer(&framebuffer_info, None)
                .ok()
        }
    }).collect::<Vec<_>>());
    self
}

  pub fn create_render_pass(&mut self) -> Result<&mut Self, vk::Result> {
    let color_attachment = vk::AttachmentDescription::builder()
      .format(self.swapchain_image_format.expect("Swapchain Image Format not set"))
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::STORE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
      .build();

    let depth_attachment = vk::AttachmentDescription::builder()
      .format(vk::Format::D32_SFLOAT)
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::DONT_CARE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
      .build();

    let color_attachment_ref = vk::AttachmentReference::builder()
      .attachment(0)
      .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
      .build();

    let depth_attachment_ref = vk::AttachmentReference::builder()
      .attachment(1)
      .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
      .build();

    let attachments = [color_attachment, depth_attachment];

    let subpass = vk::SubpassDescription::builder()
      .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
      .color_attachments(std::slice::from_ref(&color_attachment_ref))
      .depth_stencil_attachment(&depth_attachment_ref)
      .build();

    let dependency = vk::SubpassDependency::builder()
      .src_subpass(vk::SUBPASS_EXTERNAL)
      .dst_subpass(0)
      .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
      .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
      .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
      .build();

    let render_pass_info = vk::RenderPassCreateInfo::builder()
      .attachments(&attachments)
      .subpasses(std::slice::from_ref(&subpass))
      .dependencies(std::slice::from_ref(&dependency))
      .build();

    let render_pass = unsafe {
      self.logical_device.as_ref().unwrap().create_render_pass(&render_pass_info, None)?
    };

    self.render_pass = Some(render_pass);
    Ok(self)
  }

  fn query_surface_capabilities(&mut self) -> Result<&mut Self, vk::Result> {
    let physical_device = self.physical_device.expect("Physical device not initialized");
    let surface = self.surface.expect("Surface not initialzied");
    let surface_loader = self.surface_loader.as_ref().expect("Surface Loader not initialized");
    let surface_capabilities = unsafe {
      surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };

    self.surface_capabilities = Some(surface_capabilities);
    Ok(self)
  }

  fn configure_surface_format(&mut self) -> Result<&mut Self, vk::Result> {
    let physical_device = self.physical_device.expect("Physical device not initialized");
    let surface = self.surface.expect("Surface not initialzied");
    let surface_loader = self.surface_loader.as_ref().expect("Surface Loader not initialized");
    let formats = unsafe {
      surface_loader.get_physical_device_surface_formats(physical_device, surface)?
    };

    let format = if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
      vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_SRGB,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
      }
    } else {
      formats.iter()
        .find(|format| format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
        .cloned()
        .or_else(|| formats.get(0).cloned())
        .ok_or(vk::Result::ERROR_FORMAT_NOT_SUPPORTED)?
    };

    self.surface_format = Some(format);
    self.swapchain_image_format = Some(self.surface_format.as_ref().unwrap().format);
    Ok(self)
  }

  fn configure_presentation_mode(&mut self) -> Result<&mut Self, vk::Result> {
    let physical_device = self.physical_device.expect("Physical device not initialized");
    let surface = self.surface.expect("Surface not initialzied");
    let surface_loader = self.surface_loader.as_ref().expect("Surface Loader not initialized");
    let present_modes = unsafe {
      surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
    };

    let mut optimal_mode = vk::PresentModeKHR::FIFO;
    for &mode in present_modes.iter() {
      if mode == vk::PresentModeKHR::MAILBOX {
        self.presentation_mode = Some(mode);
        return Ok(self);
      } else if mode == vk::PresentModeKHR::IMMEDIATE {
        optimal_mode = vk::PresentModeKHR::IMMEDIATE;
      }
    }

    self.presentation_mode = Some(optimal_mode);
    Ok(self)
  }

  fn configure_swap_extent(&mut self, window: &winit::window::Window) -> Result<&mut Self, vk::Result> {
    if self.surface_capabilities.as_ref().unwrap().current_extent.width != u32::MAX {
      self.swap_extent = Some(self.surface_capabilities.as_ref().unwrap().current_extent);
      return Ok(self)
    } else {
      let window_size = window.inner_size();
      let extent = vk::Extent2D {
        width: window_size.width.clamp(
          self.surface_capabilities.as_ref().unwrap().min_image_extent.width, 
          self.surface_capabilities.as_ref().unwrap().max_image_extent.width
        ),
        height: window_size.height.clamp(
          self.surface_capabilities.as_ref().unwrap().min_image_extent.height,
          self.surface_capabilities.as_ref().unwrap().max_image_extent.height,
        ),
      };
      self.swap_extent = Some(extent);
    }
    Ok(self)
  }

  fn select_physical_device(&self) -> Result<vk::PhysicalDevice, vk::Result> {
    let physical_devices = unsafe { self.instance.enumerate_physical_devices()? };

    if physical_devices.is_empty() {
      return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
    }

    for &device in physical_devices.iter() {
      if VulkanInstance::check_device_compatibility(device, &self.instance) {
        return Ok(device);
      }
    }

    Err(vk::Result::ERROR_DEVICE_LOST)
  }

  fn check_device_compatibility(device: vk::PhysicalDevice, instance: &ash::Instance) -> bool {
    let properties = unsafe { instance.get_physical_device_properties(device) };
    let features = unsafe { instance.get_physical_device_features(device) };
    features.geometry_shader != 0 
  }

  fn identify_required_queue_family_indices(&self, physical_device: vk::PhysicalDevice, instance: &ash::Instance) -> Option<(u32, u32)> {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    queue_families.iter().enumerate().find_map(|(index, info)| {
      if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
        Some(index as u32)
      } else {
        None
      }
    });

    let mut graphics_index = None;
    let mut presentation_index = None;

    for (index, queue_family) in queue_families.iter().enumerate() {

      let supports_graphics = queue_family.queue_flags.contains(QueueFlags::GRAPHICS);

      let surface_support_result = unsafe {
        self.surface_loader
          .as_ref()
          .expect("VulkanInstance Surface Loader not initialized")
          .get_physical_device_surface_support(physical_device, index as u32, self.surface.expect("Surface not initialized"))
      };

      if let Ok(supports_presentation) = surface_support_result {
        if supports_graphics {
          graphics_index = Some(index as u32);
        }

        if supports_presentation {
          presentation_index = Some(index as u32);
        }
      } else {
        return None;
      }

      if graphics_index.is_some() && presentation_index.is_some() {
        break;
      }
    }

    match (graphics_index, presentation_index) {
      (Some(graphics), Some(presentation)) => Some((graphics, presentation)),
      _ => None,
    }
  }

  fn load_instance_extensions() -> Vec<*const c_char> {
    let mut extensions: Vec<*const c_char> = vec![];

    extensions.push(ash::extensions::khr::Surface::name().as_ptr());

    #[cfg(target_os = "windows")] extensions.push(ash::extensions::khr::Win32Surface::name().as_ptr());
    
    #[cfg(target_os = "linux")] extensions.push(ash::extensions::khr::XlibSurface::name().as_ptr());
    #[cfg(target_os = "linux")] extensions.push(ash::extensions::khr::XcbSurface::name().as_ptr());
    #[cfg(target_os = "linux")] extensions.push(ash::extensions::khr::WaylandSurface::name().as_ptr());

    #[cfg(target_os = "android")] extensions.push(ash::extensions::khr::AndroidSurface::name().as_ptr());
    
    extensions
  }

  fn format_device_properties(properties: vk::PhysicalDeviceProperties) -> String {
    let device_name = unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy().into_owned();
    let api_version    = properties.api_version;
    let driver_version = properties.driver_version;
    let vendor_id      = properties.vendor_id;
    let device_id      = properties.device_id;
    let device_type = format!("{:?}", properties.device_type);
    format!("Name: {}\nAPI Version: {}\nDriver Version: {}\nVendor ID: {}\nDevice ID: {}\nType: {}",
      device_name, api_version, driver_version, vendor_id, device_id, device_type
    )
  }

  fn format_device_features(features: vk::PhysicalDeviceFeatures) -> String {
    format!(
      "Geometry Shader: {}\nTesselation Shader: {}\n",
      features.geometry_shader != 0,  
      features.tessellation_shader != 0
    )
  }

  pub fn allocate_resources(&mut self, max_sets: u32) -> &mut Self {
    match self.vulkan_resources {
      None => {
        let resource_manager = VulkanResources::new(self.logical_device.as_ref().unwrap(), max_sets);
        self.vulkan_resources = Some(resource_manager);
      },
      Some(_) => panic!("VkResourceManager already bound to VulkanInstance")
    }
    self
  }

  pub fn create_command_pool(&mut self) -> &mut Self {

    let pool_create_info = CommandPoolCreateInfo::builder()
      .queue_family_index(self.graphics_queue_family_index.unwrap())
      .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
      .build();

    self.command_pool = Some(unsafe {
        self.logical_device.as_ref().unwrap().create_command_pool(&pool_create_info, None)
          .expect("Failed to create Command Pool")
      }
    );
    self
  }

  pub fn create_synchronization_objects(&mut self) {
    
    self.image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    self.render_complete_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    self.in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

    let semaphore_info = SemaphoreCreateInfo::default();
    let fence_info = FenceCreateInfo::builder()
      .flags(FenceCreateFlags::SIGNALED)
      .build();

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
      let image_available_semaphore = unsafe {
        self.logical_device.as_ref().unwrap().create_semaphore(&semaphore_info, None)
          .expect("Failed to create Image Availability Semaphore")
      };
      let render_complete_semaphore = unsafe {
        self.logical_device.as_ref().unwrap().create_semaphore(&semaphore_info, None)
          .expect("Failed to create Render Complete Semaphore")
      };
      let in_flight_fence = unsafe {
        self.logical_device.as_ref().unwrap().create_fence(&fence_info, None)
          .expect("Failed to create In-flight Fence")
      };

      self.image_available_semaphores.push(image_available_semaphore);
      self.render_complete_semaphores.push(render_complete_semaphore);
      self.in_flight_fences.push(in_flight_fence);
    }
  }

  pub fn allocate_command_buffers(&mut self) -> &mut Self {
    
    let command_buffer_count = self.swapchain_images.as_ref().unwrap().len();
    let allocate_info = CommandBufferAllocateInfo::builder()
      .command_pool(self.command_pool.unwrap())
      .level(CommandBufferLevel::PRIMARY)
      .command_buffer_count(command_buffer_count as u32)
      .build();

    self.command_buffers = unsafe {
      Some(self.logical_device.as_ref().unwrap().allocate_command_buffers(&allocate_info)
        .expect("Failed to allocate Command Buffer"))
    };

    self
  }

  pub fn define_shader(&mut self, shader_id: &str, bindings: Vec<DescriptorSetLayoutBinding>) -> &mut Self {
    self.vulkan_resources
      .as_mut()
      .unwrap()
      .create_shader_resources(shader_id)
      .new_descriptor_layout(self.logical_device.as_ref().unwrap(), shader_id, bindings)
      .allocate_shader_descriptor_sets(self.logical_device.as_ref().unwrap(), shader_id);
    self
  }
  
  pub fn create_pipeline_layout(&mut self, shader_id: &str) -> PipelineLayout {
    self.vulkan_resources
      .as_mut()
      .unwrap()
      .create_pipeline_layout(self.logical_device.as_ref().unwrap(), shader_id)
  }

  pub fn configure_graphics_pipeline(&mut self, pipeline_id: &str, pipeline_layout: vk::PipelineLayout, pipeline_config: PipelineConfig) {
    self.vulkan_resources
      .as_mut()
      .unwrap()
      .create_graphics_pipeline(
        self.logical_device.as_ref().unwrap(),
        self.render_pass.unwrap(), 
        pipeline_id,
        pipeline_layout, 
        pipeline_config,
        self.swap_extent.unwrap(),
      );
  }

  pub fn allocate_vertex_buffer(&mut self, vertices: &[Vertex]) {
    let vbo = self.vulkan_resources.as_mut().unwrap().allocate_vertex_buffer(vertices, &self.instance, self.physical_device.unwrap(), &self.logical_device.as_mut().unwrap());
    self.vertex_count = vertices.len();
    self.vertex_buffer        = Some(vbo.0);
    self.vertex_buffer_memory = Some(vbo.1);
  }

  pub fn acquire_next_image_index(&self, semaphore_index: usize) -> Result<u32, vk::Result> {
    let timeout = u64::MAX;
    let semaphore = self.image_available_semaphores[semaphore_index];
    let (image_index, _is_suboptimal) = unsafe {
      self.swapchain_loader.as_ref().unwrap().acquire_next_image(
        self.swapchain.unwrap(), 
        timeout, 
        semaphore, 
        Fence::null()
      )?
    };

    Ok(image_index)
  }

  pub fn record_command_buffer(&mut self, pipeline_id: &str, image_index: usize) {
    
    let command_buffer = self.command_buffers.as_ref().unwrap()[image_index];
    let begin_info = CommandBufferBeginInfo::builder()
      .flags(CommandBufferUsageFlags::SIMULTANEOUS_USE)
      .build();

    unsafe {
      self.logical_device.as_ref().unwrap().begin_command_buffer(command_buffer, &begin_info)
        .expect("Failed to begin recording command buffer");

      // Begin Render Pass
      let render_pass_begin_info = RenderPassBeginInfo::builder()
        .render_pass(self.render_pass.unwrap())
        .framebuffer(self.swapchain_framebuffers.as_ref().unwrap()[image_index])
        .render_area(Rect2D {
          offset: Offset2D { x: 0, y: 0 },
          extent: self.swap_extent.unwrap()
        })
        .clear_values(&[ClearValue {
          color: ClearColorValue {
            float32: [1.0, 0.0, 0.0, 1.0], // Clear Value
          },
        }])
        .build();

      self.logical_device.as_ref().unwrap().cmd_begin_render_pass(
        command_buffer, 
        &render_pass_begin_info,
        SubpassContents::INLINE
      );

      // Bind Pipeline
      self.logical_device.as_ref().unwrap().cmd_bind_pipeline(
        command_buffer, 
        PipelineBindPoint::GRAPHICS, 
        self.vulkan_resources.as_mut().unwrap().get_graphics_pipeline(pipeline_id)
      );

      let vertex_buffers = [self.vertex_buffer.unwrap()];
      let offsets = [0];
      unsafe {
        self.logical_device.as_ref().unwrap().cmd_bind_vertex_buffers(
          command_buffer, 
          0, 
          &vertex_buffers, 
          &offsets
        );

        println!("Debug draw");
        
        /* Draw */
        self.logical_device.as_ref().unwrap().cmd_draw(
          command_buffer, 
          self.vertex_count as u32, 
          2, 
          0, 
          0
        )
      }
    }
  }

  pub fn get_image_in_flight(&mut self, image_index: usize) -> Option<Fence> {
    self.images_in_flight[image_index]
  }
}