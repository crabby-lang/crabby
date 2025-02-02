use ash::{version::InstanceV1_0, vk, Entry, Instance};
use std::ptr;

pub struct Graphics {
    entry: Entry,
    instance: Instance,
}

impl Graphics {
    pub fn new(app_name: &str) -> Self {
        let entry = unsafe { Entry::linked() }.unwrap();

        let app_info = vk::Application {
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default() 
        };

        let instance = unsafe { entry.create_instance(&create_info, None) }.unwrap();

        Self { entry, instance }
    }
}
