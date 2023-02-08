use std::ffi::CStr;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;
#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;
use ash::vk;

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSView, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::id as cocoa_id;
#[cfg(target_os = "macos")]
use metal::CoreAnimationLayer;
#[cfg(target_os = "macos")]
use objc::runtime::YES;

// extensions ----------
#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<&'static CStr> {
    let mut request = vec![Surface::name(), MacOSSurface::name()];
    if enable_debug {
        request.push(DebugUtils::name());
    }
    request
}

#[cfg(target_os = "windows")]
pub fn required_extension_names(enable_debug: bool) -> Vec<&'static CStr> {
    let mut request = vec![Surface::name(), Win32Surface::name()];
    if enable_debug {
        request.push(DebugUtils::name());
    }
    request
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_extension_names(enable_debug: bool) -> Vec<&'static CStr> {
    let mut request = vec![Surface::name(), XlibSurface::name()];
    if enable_debug {
        request.push(DebugUtils::name());
    }
    request
}
