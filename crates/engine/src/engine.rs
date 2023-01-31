use math::Mat4;
use rhi::vulkan::renderer::VulkanRenderer;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Engine {
    pub(crate) renderer: Rc<RefCell<VulkanRenderer>>,
}

impl Engine {
    pub fn renderer_set_view(&mut self, view: Mat4) {
        self.renderer.borrow_mut().set_view(view);
    }
}
