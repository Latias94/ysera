use imgui::TextureId;

use math::{vec2, Vec2};

#[derive(Clone)]
pub struct GuiState {
    pub hovered: bool,
    pub value: f32,
    pub opacity: f32,
    pub fovy: f32,
    pub viewport_xy: Vec2,
    pub viewport_size: Vec2,
    pub open_demo_window: bool,
    pub test_texture_id: Option<TextureId>,
}

impl GuiState {
    pub fn new(viewport_size: Vec2, test_texture_id: Option<TextureId>) -> Self {
        Self {
            hovered: false,
            value: 0f32,
            opacity: 1f32,
            fovy: 45f32,
            viewport_xy: vec2(0.0, 0.0),
            viewport_size,
            open_demo_window: false,
            test_texture_id,
        }
    }
}

pub fn draw_imgui(state: &mut GuiState, ui: &mut imgui::Ui) {
    ui.window("Menu")
        // .collapsed(true, Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .size([220.0, 220.0], imgui::Condition::FirstUseEver)
        .focus_on_appearing(false)
        .bg_alpha(0.9f32)
        // .movable(false)
        .build(|| {
            ui.slider("rotate", 0f32, 360f32, &mut state.value);
            ui.slider("opacity", 0f32, 1f32, &mut state.opacity);
            ui.slider("fovy", 0f32, 90f32, &mut state.fovy);
            {
                let token = ui.push_item_width(80f32);
                ui.slider(
                    "x",
                    -state.viewport_size.x / 2f32,
                    state.viewport_size.x / 2f32,
                    &mut state.viewport_xy.x,
                );
                ui.same_line();
                ui.slider(
                    "y",
                    -state.viewport_size.y / 2f32,
                    state.viewport_size.y / 2f32,
                    &mut state.viewport_xy.y,
                );
                token.end();
            }

            ui.checkbox("open demo window", &mut state.open_demo_window);
            if state.open_demo_window {
                ui.show_demo_window(&mut state.open_demo_window);
            }
        });
    ui.window("Viewport")
        // .collapsed(true, Condition::FirstUseEver)
        .position([0.0, 220.0], imgui::Condition::FirstUseEver)
        .size([220.0, 220.0], imgui::Condition::FirstUseEver)
        .focus_on_appearing(false)
        .bg_alpha(1f32)
        // .movable(false)
        .build(|| {
            ui.text("Hello textures!");
            if let Some(my_texture_id) = state.test_texture_id {
                ui.text("Some generated texture");
                imgui::Image::new(my_texture_id, [100.0, 100.0]).build(ui);
            }
        });
    state.hovered = ui.is_any_item_hovered()
        || ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::ANY_WINDOW);
}
