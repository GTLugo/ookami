use std::sync::Arc;

use thiserror::Error;
use tracing::{error, warn};
use vulkano::{
  command_buffer::CommandBufferExecError, instance::{
    debug::{
      DebugUtilsMessageSeverity,
      DebugUtilsMessageType,
      DebugUtilsMessenger,
      DebugUtilsMessengerCallback,
      DebugUtilsMessengerCreateInfo,
    },
    Instance,
  }, Validated
};

use super::instance::FoxyInstance;

#[derive(Error, Debug)]
pub enum VulkanError {
  #[error("{0}")]
  Error(String),
  #[error("{0}")]
  VulkanoError(#[from] vulkano::VulkanError),
  #[error("{0}")]
  ValidatedVulkanoError(#[from] Validated<vulkano::VulkanError>),
  #[error("{0}")]
  AllocateImageError(#[from] Validated<vulkano::image::AllocateImageError>),
  #[error("{0}")]
  LoadingError(#[from] vulkano::LoadingError),
  #[error("{0}")]
  ValidationError(#[from] Box<vulkano::ValidationError>),
  #[error("{0}")]
  CommandBufferExecError(#[from] CommandBufferExecError),
  #[error("{0}")]
  Shaderc(#[from] shaderc::Error),
  #[error("{0}")]
  Unsupported(String),
  #[error("no supported device found")]
  NoValidDevice,
  #[error("{0}")]
  Shader(String),
  #[error("{0}")]
  SyncObjects(String),
  #[error("{0}")]
  IO(#[from] std::io::Error),
  #[error("surface suboptimal")]
  Suboptimal,
  #[error("expected different shaders for pipeline")]
  MismatchedShaders,
}

#[macro_export]
macro_rules! vulkan_unsupported_error {
  () => {
    $crate::vulkan::error::VulkanError::Unsupported(format!("attempted action unsupported by the device running Vulkan"))
  };
  ($($arg:tt)*) => {{
    $crate::vulkan::error::VulkanError::Unsupported(format!($($arg)*))
  }}
}

#[macro_export]
macro_rules! vulkan_shader_error {
  () => {
    $crate::vulkan::error::VulkanError::Unsupported(format!("attempted action unsupported by the device running Vulkan"))
  };
  ($($arg:tt)*) => {{
    $crate::vulkan::error::VulkanError::Unsupported(format!($($arg)*))
  }}
}

#[macro_export]
macro_rules! vulkan_error {
  () => {
    $crate::vulkan::error::VulkanError::Error(format!("vulkan error"))
  };
  ($($arg:tt)*) => {{
    $crate::vulkan::error::VulkanError::Error(format!($($arg)*))
  }}
}

pub struct Debug {
  _debug: Option<DebugUtilsMessenger>,
}

impl Debug {
  pub fn new(instance: Arc<Instance>) -> Result<Arc<Self>, VulkanError> {
    if FoxyInstance::ENABLE_VALIDATION_LAYERS {
      let debug = DebugUtilsMessenger::new(instance, DebugUtilsMessengerCreateInfo {
        message_severity: DebugUtilsMessageSeverity::ERROR | DebugUtilsMessageSeverity::WARNING,
        message_type: DebugUtilsMessageType::VALIDATION | DebugUtilsMessageType::PERFORMANCE,
        ..DebugUtilsMessengerCreateInfo::user_callback(unsafe {
          DebugUtilsMessengerCallback::new(|sev, ty, data| {
            let ty = if ty.intersects(DebugUtilsMessageType::GENERAL) {
              "General"
            } else if ty.intersects(DebugUtilsMessageType::VALIDATION) {
              "Validation"
            } else {
              "Performance"
            };

            let msg = format!("Vulkan {ty}: {:?}", data.message);

            match sev {
              DebugUtilsMessageSeverity::ERROR => error!(msg),
              DebugUtilsMessageSeverity::WARNING => warn!(msg),
              _ => (),
            }
          })
        })
      })?;
      Ok(Arc::new(Self { _debug: Some(debug) }))
    } else {
      Ok(Arc::new(Self { _debug: None }))
    }
  }
}