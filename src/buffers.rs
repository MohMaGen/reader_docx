#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum BufferName {
    ConsoleBg
}

pub struct UiBuffers {
    pub console_bg: wgpu::Buffer,
}

impl crate::draw::DrawState<'_> {
    pub fn get_buffer(&self, name: BufferName) -> &wgpu::Buffer {
        match name {
            BufferName::ConsoleBg => &self.ui_buffers.console_bg
        }
    }
}
