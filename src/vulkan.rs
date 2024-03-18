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
  pub logical_device: Option<ash::Device>,
  pub surface: Option<SurfaceKHR>,
  surface_loader: Option<Surface>,
  graphics_queue_family_index: Option<u32>,
  presentation_queue_family_index: Option<u32>,
  graphics_queue: Option<vk::Queue>,
}

impl VulkanInstance {
  
  pub fn new(app_name: &str, engine_name: &str) -> Result<Self, vk::Result> {

    let entry = unsafe { match Entry::load() {
      Ok(entry) => entry,
      Err(_)    => return Err(vk::Result::ERROR_INITIALIZATION_FAILED),
    } };

    println!("Initializing Vulkan Instance");

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
      logical_device: None,
      surface: None,
      surface_loader: None,
      graphics_queue_family_index: None,
      presentation_queue_family_index: None,
      graphics_queue: None
    })
  }

  pub unsafe fn create_surface(&mut self, window: &Window) -> Result<&mut Self, vk::Result> {

    if (self.surface_loader.is_none()) {
      self.surface_loader = Some(Surface::new(&self._entry, &self.instance));
    }

    let raw_window_handle   = window.raw_window_handle();
    let raw_display_handle = window.raw_display_handle();

    /* Win32
    let surface = match raw_window_handle {
      RawWindowHandle::Win32(_) => {
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
    };
    */

    // Platform Agnostic
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

  pub fn configure_hardware(&mut self) -> &mut Self{
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

    println!("\nPhysical Device Properties:\n{}", VulkanInstance::format_device_properties(properties));
    println!("\nPhysical Device Features:\n{}", VulkanInstance::format_device_features(features));

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

  pub fn create_logical_device(&mut self) -> Result<(), vk::Result> {
    let queue_priorities = [1.0_f32];
    let queue_family_index = self.graphics_queue_family_index.unwrap();

    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
      .queue_family_index(queue_family_index)
      .queue_priorities(&queue_priorities)
      .build();

    let physical_device_features = vk::PhysicalDeviceFeatures::default();

    let device_create_info = vk::DeviceCreateInfo::builder()
      .queue_create_infos(&[queue_create_info])
      .enabled_features(&physical_device_features)
      .build();

    let logical_device = unsafe {
      self.instance.create_device(self.physical_device.unwrap(), &device_create_info, None)?
    };

    self.logical_device = Some(logical_device);

    let graphics_queue = unsafe {
      self.logical_device.as_ref().unwrap().get_device_queue(queue_family_index, 0)
    };

    self.graphics_queue = Some(graphics_queue);

    Ok(())
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
          .expect("VulkanInstance=>surface_loader not initialized")
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

  fn format_device_properties(properties: vk::PhysicalDeviceProperties) -> String {
    let device_name = unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy().into_owned();
    let api_version = properties.api_version;
    let driver_version = properties.driver_version;
    let vendor_id = properties.vendor_id;
    let device_id = properties.device_id;
    let device_type = format!("{:?}", properties.device_type);
    format!("Device Name: {}\nAPI Version: {}\nDriver Version: {}\nVendor ID: {}\nDevice ID: {} Device Type: {}",
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

}