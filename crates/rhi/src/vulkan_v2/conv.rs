use crate::types_v2::{
    RHIAccessFlags, RHIAttachmentDescriptionFlags, RHIAttachmentLoadOp, RHIAttachmentReference,
    RHIAttachmentStoreOp, RHICommandPoolCreateFlags, RHIFormat, RHIImageLayout,
    RHIPipelineBindPoint, RHIPipelineStageFlags, RHIRenderPassCreateFlags, RHISampleCountFlagBits,
    RHISubpassDescriptionFlags,
};
use ash::vk;
use num_traits::{FromPrimitive, ToPrimitive};

impl RHICommandPoolCreateFlags {
    pub fn to_vk(&self) -> vk::CommandPoolCreateFlags {
        let mut flags = vk::CommandPoolCreateFlags::empty();
        if self.contains(RHICommandPoolCreateFlags::TRANSIENT) {
            flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }
        if self.contains(RHICommandPoolCreateFlags::RESET_COMMAND_BUFFER) {
            flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }
        flags
    }
}

impl From<vk::Format> for RHIFormat {
    fn from(value: vk::Format) -> Self {
        match RHIFormat::from_i32(value.as_raw()) {
            None => RHIFormat::UNDEFINED,
            Some(x) => x,
        }
    }
}

impl RHIFormat {
    pub fn to_vk(self) -> vk::Format {
        match self.to_i32() {
            None => vk::Format::UNDEFINED,
            Some(x) => vk::Format::from_raw(x),
        }
    }
}

impl RHIAttachmentDescriptionFlags {
    pub fn to_vk(&self) -> vk::AttachmentDescriptionFlags {
        let mut flags = vk::AttachmentDescriptionFlags::empty();
        if self.contains(RHIAttachmentDescriptionFlags::MAY_ALIAS) {
            flags |= vk::AttachmentDescriptionFlags::MAY_ALIAS;
        }
        flags
    }
}

impl RHISubpassDescriptionFlags {
    pub fn to_vk(&self) -> vk::SubpassDescriptionFlags {
        let mut flags = vk::SubpassDescriptionFlags::empty();
        flags
    }
}

impl RHISampleCountFlagBits {
    pub fn to_vk(self) -> vk::SampleCountFlags {
        match self.to_u32() {
            None => vk::SampleCountFlags::TYPE_1,
            Some(x) => vk::SampleCountFlags::from_raw(x),
        }
    }
}

impl RHIAttachmentLoadOp {
    pub fn to_vk(self) -> vk::AttachmentLoadOp {
        match self.to_i32() {
            None => vk::AttachmentLoadOp::DONT_CARE,
            Some(x) => vk::AttachmentLoadOp::from_raw(x),
        }
    }
}

impl RHIAttachmentStoreOp {
    pub fn to_vk(self) -> vk::AttachmentStoreOp {
        match self.to_i32() {
            None => vk::AttachmentStoreOp::DONT_CARE,
            Some(x) => vk::AttachmentStoreOp::from_raw(x),
        }
    }
}

impl RHIImageLayout {
    pub fn to_vk(self) -> vk::ImageLayout {
        match self.to_i32() {
            None => vk::ImageLayout::UNDEFINED,
            Some(x) => vk::ImageLayout::from_raw(x),
        }
    }
}

impl RHIPipelineBindPoint {
    pub fn to_vk(self) -> vk::PipelineBindPoint {
        match self.to_i32() {
            None => vk::PipelineBindPoint::GRAPHICS,
            Some(x) => vk::PipelineBindPoint::from_raw(x),
        }
    }
}

impl RHIAttachmentReference {
    pub fn to_vk(self) -> vk::AttachmentReference {
        vk::AttachmentReference::builder()
            .attachment(self.attachment)
            .layout(self.layout.to_vk())
            .build()
    }
}

impl RHIPipelineStageFlags {
    pub fn to_vk(&self) -> vk::PipelineStageFlags {
        let mut flags = vk::PipelineStageFlags::empty();
        if self.contains(RHIPipelineStageFlags::TOP_OF_PIPE) {
            flags |= vk::PipelineStageFlags::TOP_OF_PIPE;
        }
        if self.contains(RHIPipelineStageFlags::DRAW_INDIRECT) {
            flags |= vk::PipelineStageFlags::DRAW_INDIRECT;
        }
        if self.contains(RHIPipelineStageFlags::VERTEX_INPUT) {
            flags |= vk::PipelineStageFlags::VERTEX_INPUT;
        }
        if self.contains(RHIPipelineStageFlags::VERTEX_SHADER) {
            flags |= vk::PipelineStageFlags::VERTEX_SHADER;
        }
        if self.contains(RHIPipelineStageFlags::TESSELLATION_CONTROL_SHADER) {
            flags |= vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER;
        }
        if self.contains(RHIPipelineStageFlags::TESSELLATION_EVALUATION_SHADER) {
            flags |= vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER;
        }
        if self.contains(RHIPipelineStageFlags::GEOMETRY_SHADER) {
            flags |= vk::PipelineStageFlags::GEOMETRY_SHADER;
        }
        if self.contains(RHIPipelineStageFlags::FRAGMENT_SHADER) {
            flags |= vk::PipelineStageFlags::FRAGMENT_SHADER;
        }
        if self.contains(RHIPipelineStageFlags::EARLY_FRAGMENT_TESTS) {
            flags |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
        }
        if self.contains(RHIPipelineStageFlags::LATE_FRAGMENT_TESTS) {
            flags |= vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
        }
        if self.contains(RHIPipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) {
            flags |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        }
        if self.contains(RHIPipelineStageFlags::COMPUTE_SHADER) {
            flags |= vk::PipelineStageFlags::COMPUTE_SHADER;
        }
        if self.contains(RHIPipelineStageFlags::TRANSFER) {
            flags |= vk::PipelineStageFlags::TRANSFER;
        }
        if self.contains(RHIPipelineStageFlags::BOTTOM_OF_PIPE) {
            flags |= vk::PipelineStageFlags::BOTTOM_OF_PIPE;
        }
        if self.contains(RHIPipelineStageFlags::HOST) {
            flags |= vk::PipelineStageFlags::HOST;
        }
        if self.contains(RHIPipelineStageFlags::ALL_GRAPHICS) {
            flags |= vk::PipelineStageFlags::ALL_GRAPHICS;
        }
        if self.contains(RHIPipelineStageFlags::ALL_COMMANDS) {
            flags |= vk::PipelineStageFlags::ALL_COMMANDS;
        }
        flags
    }
}

impl RHIAccessFlags {
    pub fn to_vk(&self) -> vk::AccessFlags {
        let mut flags = vk::AccessFlags::empty();
        if self.contains(RHIAccessFlags::INDIRECT_COMMAND_READ) {
            flags |= vk::AccessFlags::INDIRECT_COMMAND_READ;
        }
        if self.contains(RHIAccessFlags::INDEX_READ) {
            flags |= vk::AccessFlags::INDEX_READ;
        }
        if self.contains(RHIAccessFlags::VERTEX_ATTRIBUTE_READ) {
            flags |= vk::AccessFlags::VERTEX_ATTRIBUTE_READ;
        }
        if self.contains(RHIAccessFlags::UNIFORM_READ) {
            flags |= vk::AccessFlags::UNIFORM_READ;
        }
        if self.contains(RHIAccessFlags::INPUT_ATTACHMENT_READ) {
            flags |= vk::AccessFlags::INPUT_ATTACHMENT_READ;
        }
        if self.contains(RHIAccessFlags::SHADER_READ) {
            flags |= vk::AccessFlags::SHADER_READ;
        }
        if self.contains(RHIAccessFlags::SHADER_WRITE) {
            flags |= vk::AccessFlags::SHADER_WRITE;
        }
        if self.contains(RHIAccessFlags::COLOR_ATTACHMENT_READ) {
            flags |= vk::AccessFlags::COLOR_ATTACHMENT_READ;
        }
        if self.contains(RHIAccessFlags::COLOR_ATTACHMENT_WRITE) {
            flags |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        }
        if self.contains(RHIAccessFlags::DEPTH_STENCIL_ATTACHMENT_READ) {
            flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ;
        }
        if self.contains(RHIAccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE) {
            flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        }
        if self.contains(RHIAccessFlags::TRANSFER_READ) {
            flags |= vk::AccessFlags::TRANSFER_READ;
        }
        if self.contains(RHIAccessFlags::TRANSFER_WRITE) {
            flags |= vk::AccessFlags::TRANSFER_WRITE;
        }
        if self.contains(RHIAccessFlags::HOST_READ) {
            flags |= vk::AccessFlags::HOST_READ;
        }
        if self.contains(RHIAccessFlags::HOST_WRITE) {
            flags |= vk::AccessFlags::HOST_WRITE;
        }
        if self.contains(RHIAccessFlags::MEMORY_READ) {
            flags |= vk::AccessFlags::MEMORY_READ;
        }
        if self.contains(RHIAccessFlags::MEMORY_WRITE) {
            flags |= vk::AccessFlags::MEMORY_WRITE;
        }
        flags
    }
}

impl RHIRenderPassCreateFlags {
    pub fn to_vk(&self) -> vk::RenderPassCreateFlags {
        let mut flags = vk::RenderPassCreateFlags::empty();
        flags
    }
}
