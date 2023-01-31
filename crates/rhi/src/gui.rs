use imgui::TextureId;

// todo wait for refactor render system
#[derive(Clone)]
pub struct GuiState {
    pub hovered: bool,
    // pub value: f32,
    // pub opacity: f32,
    // pub fovy: f32,
    // pub near_clip: f32,
    // pub far_clip: f32,
    pub open_demo_window: bool,
    pub test_texture_id: Option<TextureId>,
}

impl GuiState {
    pub fn new(test_texture_id: Option<TextureId>) -> Self {
        Self {
            hovered: false,
            // value: 0f32,
            // opacity: 1f32,
            // fovy: 45f32,
            // near_clip: 0.1,
            // far_clip: 10.0,
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
            // ui.slider("rotate", 0f32, 360f32, &mut state.value);
            // ui.slider("opacity", 0f32, 1f32, &mut state.opacity);
            // ui.slider("fovy", 0f32, 90f32, &mut state.fovy);
            // {
            //     let token = ui.push_item_width(120f32);
            //     ui.slider("near_clip", 0.1, 10.0, &mut state.near_clip);
            //     // ui.same_line();
            //     ui.slider("far_clip", 10.0, 100.0, &mut state.far_clip);
            //     token.end();
            // }

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
