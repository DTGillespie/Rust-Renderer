use std::ffi::CString;
use std::os::raw::c_char;
use ash::{
  vk, 
  vk::QueueFlags, 
  vk::SurfaceKHR,
  Entry, 
  extensions::khr::Surface, 
};
use winit::{
  event_loop::EventLoopWindowTarget,
  window::Window,
};
use raw_window_handle::{
  HasRawWindowHandle, 
  RawWindowHandle,
  HasRawDisplayHandle,
  RawDisplayHandle,
};

pub struct VulkanInstance {
  _entry: Entry,
  pub instance: ash::Instance,
  pub physical_device: Option<vk::PhysicalDevice>,
  pub surface: Option<SurfaceKHR>,
  surface_loader: Option<Surface>,
}

impl VulkanInstance {
  
  pub fn new(app_name: &str, engine_name: &str) -> Result<Self, vk::Result> {

    let entry = unsafe { match Entry::load() {
      Ok(entry) => entry,
      Err(_)    => return Err(vk::Result::ERROR_INITIALIZATION_FAILED),
    } };

    println!("Initializing Vulkan instance");

    let app_name_cstr = CString::new(app_name).unwrap();
    let engine_name_cstr = CString::new(engine_name).unwrap();
    
    let app_info = vk::ApplicationInfo::builder()
      .application_name(&app_name_cstr)
      .application_version(vk::make_api_version(0, 0, 0, 0))
      .engine_name(&engine_name_cstr)
      .engine_version(vk::make_api_version(0, 0, 0, 0))
      .api_version(vk::API_VERSION_1_0);

    let extension_names = VulkanInstance::get_required_extensions();
    let create_info = vk::InstanceCreateInfo::builder()
      .application_info(&app_info)
      .enabled_extension_names(&extension_names);

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    Ok(VulkanInstance {
      _entry: entry, 
      instance,
      physical_device: None,
      surface: None,
      surface_loader: None
    })
  }

  pub fn configure_hardware(&mut self) {
    let device = self.select_physical_device();
    self.physical_device = match device {
      Ok(device) => Some(device),
      Err(_) => {
        panic!("Failed to identify compatible physical device");
      }
    };

    let physical_device = match self.physical_device {
      Some(device) => device,
      None => {
        panic!("Error referencing VulkanInstance.physical_device");
      }
    };

    let graphics_queue_family_index = self.identify_required_queue_family_indices(physical_device, &self.instance);
    match graphics_queue_family_index {
      Some((index1, index2)) => {
        println!("Identified duel graphics/presentation queue family indices: {},{}", index1, index2);
      },
      None => {panic!("Failed to identify suitable graphics/presentation queue family index")}
    }

  }

  pub unsafe fn create_surface(
    &mut self,
    /*event_loop: &EventLoopWindowTarget<()>,*/
    window: &Window,
  ) -> Result<SurfaceKHR, vk::Result> {

    let raw_window_handle   = window.raw_window_handle();
    let raw_display_handle = window.raw_display_handle();
    match raw_window_handle {
      RawWindowHandle::Win32(handle) => {
        let surface = ash_window::create_surface(
          &self._entry,
          &self.instance,
          raw_display_handle,
          raw_window_handle,
          None
        ).expect("Failed to create Vulkan surface");
        Ok(surface)
      },
      _ => Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
    }
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

    features.geometry_shader != 0 && 
      properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
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
          .expect("Surface loader not initialized")
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
        println!("Failed to query physical device surface support");
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

  fn get_required_extensions() -> Vec<*const c_char> {
    let mut extensions: Vec<*const c_char> = vec![];

    // Always required: VK_KHR_surface
    extensions.push(ash::extensions::khr::Surface::name().as_ptr());

    #[cfg(target_os = "windows")]
    extensions.push(ash::extensions::khr::Win32Surface::name().as_ptr());
    
    #[cfg(target_os = "linux")]
    extensions.push(ash::extensions::khr::XlibSurface::name().as_ptr());

    #[cfg(target_os = "android")]
    extensions.push(ash::extensions::khr::AndroidSurface::name().as_ptr());
  
    extensions
  }

}