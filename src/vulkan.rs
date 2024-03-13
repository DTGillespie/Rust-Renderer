use ash::vk::QueueFlags;
use ash::{vk, Entry};
use std::ffi::CString;
use std::os::raw::c_char;

pub struct VulkanInstance {
  _entry: Entry,
  pub instance: ash::Instance,
  pub physical_device: Option<vk::PhysicalDevice>,
}

impl VulkanInstance {
  
  pub fn new(app_name: &str, engine_name: &str) -> Result<Self, vk::Result> {

    let entry = unsafe { match Entry::load() {
      Ok(entry) => entry,
      Err(_)    => return Err(vk::Result::ERROR_INITIALIZATION_FAILED),
    } };

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
    })
  }

  pub fn configure_hardware(&mut self) {
    let device = self.select_physical_device();
    self.physical_device = match device {
      Ok(device) => Some(device),
      Err(error) => {
        println!("Failed to select physical device: {:?}", error);
        None
      }
    };
  }

  fn select_physical_device(&self) -> Result<vk::PhysicalDevice, vk::Result> {
    let physical_devices = unsafe { self.instance.enumerate_physical_devices()? };

    if physical_devices.is_empty() {
      return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
    };

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
    
    let mut graphics_queue_support = false;
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
    for queue_family in queue_families {
      if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
        graphics_queue_support = true;
        break;
      }
    }

    features.geometry_shader != 0 && 
      properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU &&
      graphics_queue_support
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