pub trait RendererApi {
    fn init();
    fn shutdown();
    fn begin_frame();
    fn end_frame();

    fn begin_render_pass();
    fn end_render_pass();
}
