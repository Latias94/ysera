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

// surface ----------
// create with winit
#[cfg(target_os = "windows")]
pub unsafe fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::os::raw::c_void;
    use winit::platform::windows::WindowExtWindows;
    let hwnd = window.hwnd() as *const c_void;
    let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap();
    let hinstance = hinstance.0 as *const c_void;
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR::builder()
        .hinstance(hinstance)
        .hwnd(hwnd)
        .build();
    let win32_surface_loader = Win32Surface::new(entry, instance);
    win32_surface_loader.create_win32_surface(&win32_create_info, None)
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub unsafe fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::ptr;
    use winit::platform::unix::WindowExtUnix;

    let x11_display = window.xlib_display().unwrap();
    let x11_window = window.xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR::builder()
        .window(x11_window as vk::Window)
        .dpy(x11_display as *mut vk::Display)
        .build();
    let xlib_surface_loader = XlibSurface::new(entry, instance);
    xlib_surface_loader.create_xlib_surface(&x11_create_info, None)
}

#[cfg(target_os = "macos")]
pub unsafe fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::mem;
    use std::os::raw::c_void;
    use std::ptr;
    use winit::platform::macos::WindowExtMacOS;

    let wnd: cocoa_id = mem::transmute(window.ns_window());

    let layer = CoreAnimationLayer::new();

    layer.set_edge_antialiasing_mask(0);
    layer.set_presents_with_transaction(false);
    layer.remove_all_animations();

    let view = wnd.contentView();

    layer.set_contents_scale(view.backingScaleFactor());
    view.setLayer(mem::transmute(layer.as_ref()));
    view.setWantsLayer(YES);

    let create_info = vk::MacOSSurfaceCreateInfoMVK::builder()
        .p_view(window.ns_view() as *const c_void)
        .build();

    let macos_surface_loader = MacOSSurface::new(entry, instance);
    macos_surface_loader.create_mac_os_surface_mvk(&create_info, None)
}
