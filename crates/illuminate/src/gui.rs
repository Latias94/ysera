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
}

impl GuiState {
    pub fn new(viewport_size: Vec2) -> Self {
        Self {
            hovered: false,
            value: 0f32,
            opacity: 1f32,
            fovy: 45f32,
            viewport_xy: vec2(0.0, 0.0),
            viewport_size,
            open_demo_window: false,
        }
    }
}

pub fn draw_imgui(state: &mut GuiState, ui: &mut imgui::Ui) {
    // let choices = ["test test this is 1", "test test this is 2"];

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

            // ui.text_wrapped("Hello world!");
            // ui.text_wrapped("こんにちは世界！");
            // if ui.button(choices[state.value]) {
            //     state.value += 1;
            //     state.value %= 2;
            // }
            // ui.button("This...is...imgui-rs!");
            // ui.separator();
            // let mouse_pos = ui.io().mouse_pos;
            // ui.text(format!(
            //     "Mouse Position: ({:.1},{:.1})",
            //     mouse_pos[0], mouse_pos[1]
            // ));
        });
    state.hovered = ui.is_any_item_hovered()
        || ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::ANY_WINDOW);
}
