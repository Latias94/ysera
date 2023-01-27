use crate::support;
use imgui::{
    Condition, Context, DrawData, FontConfig, FontGlyphRanges, FontSource, WindowHoveredFlags,
};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use winit::event::Event;
use winit::window::Window as WinitWindow;

pub struct GuiContext {
    context: Context,
    winit_platform: WinitPlatform,
    instant: Instant,
    state: GuiState,
}

pub struct GuiContextDescriptor<'a> {
    pub window: &'a WinitWindow,
    pub hidpi_factor: f64,
}

impl GuiContext {
    pub fn new(desc: &GuiContextDescriptor) -> Self {
        let (context, winit_platform) = init_imgui(desc.window);

        Self {
            context,
            winit_platform,
            instant: Instant::now(),
            state: Default::default(),
        }
    }

    pub fn handle_event(&mut self, window: &WinitWindow, event: &Event<()>) {
        let io = self.context.io_mut();
        let platform = &mut self.winit_platform;

        platform.handle_event(io, window, event);
    }

    pub fn update_delta_time(&mut self) {
        let io = self.context.io_mut();
        let now = Instant::now();
        io.update_delta_time(now - self.instant);
        self.instant = now;
    }

    pub fn prepare_frame(&mut self, window: &WinitWindow) {
        let io = self.context.io_mut();
        let platform = &mut self.winit_platform;
        platform.prepare_frame(io, window).unwrap();
    }

    pub fn render(&mut self, window: &WinitWindow) -> &DrawData {
        let ui = self.context.frame();

        {
            let ui = &ui;

            let choices = ["test test this is 1", "test test this is 2"];

            ui.window("Menu")
                // .collapsed(true, Condition::FirstUseEver)
                .position([0.0, 0.0], Condition::Always)
                .size([220.0, 250.0], Condition::FirstUseEver)
                .focus_on_appearing(false)
                .movable(false)
                .bg_alpha(0.3)
                .build(|| {
                    ui.text_wrapped("Hello world!");
                    ui.text_wrapped("こんにちは世界！");
                    if ui.button(choices[self.state.value]) {
                        self.state.value += 1;
                        self.state.value %= 2;
                    }
                    ui.button("This...is...imgui-rs!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });
            self.state.hovered = ui.is_any_item_hovered()
                || ui.is_window_hovered_with_flags(WindowHoveredFlags::ANY_WINDOW);
        }

        self.winit_platform.prepare_render(ui, window);
        self.context.render()
    }

    pub fn get_context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn is_hovered(&self) -> bool {
        self.state.hovered
    }
}

fn init_imgui(window: &WinitWindow) -> (Context, WinitPlatform) {
    log::info!("Preparing imgui!");

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../../../resources/fonts/Roboto-Regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 2.0,
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../../../resources/fonts/mplus-1p-regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
    ]);

    if let Some(backend) = support::clipboard::init() {
        imgui.set_clipboard_backend(backend);
    } else {
        log::error!("Failed to initialize clipboard support");
    }

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);

    (imgui, platform)
}

#[derive(Clone)]
pub struct GuiState {
    hovered: bool,
    value: usize,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            hovered: false,
            value: 0,
        }
    }
}
