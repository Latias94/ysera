use imgui::StyleColor;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum GuiTheme {
    Default,
    Dark,
}

impl Default for GuiTheme {
    fn default() -> Self {
        GuiTheme::Default
    }
}

pub fn set_theme(theme: GuiTheme, style: &mut imgui::Style) {
    if theme == GuiTheme::Dark {
        set_dark_theme(style);
    }
}

/// https://github.com/ocornut/imgui/issues/707#issuecomment-917151020
pub fn set_dark_theme(style: &mut imgui::Style) {
    style.window_padding = [8.00, 8.00];
    style.frame_padding = [5.00, 2.00];
    style.cell_padding = [6.00, 6.00];
    style.item_spacing = [6.00, 6.00];
    style.item_inner_spacing = [6.00, 6.00];
    style.touch_extra_padding = [0.00, 0.00];

    style.indent_spacing = 25.0;
    style.scrollbar_size = 15.0;
    style.grab_min_size = 10.0;
    style.window_border_size = 1.0;
    style.child_border_size = 1.0;
    style.popup_border_size = 1.0;
    style.frame_border_size = 1.0;
    style.tab_border_size = 1.0;
    style.window_rounding = 7.0;
    style.child_rounding = 4.0;
    style.frame_rounding = 3.0;
    style.popup_rounding = 4.0;
    style.scrollbar_rounding = 9.0;
    style.grab_rounding = 3.0;
    style.log_slider_deadzone = 4.0;
    style.tab_rounding = 4.0;

    style.colors[StyleColor::Text as usize] = [1.00, 1.00, 1.00, 1.00];
    style.colors[StyleColor::TextDisabled as usize] = [0.50, 0.50, 0.50, 1.00];
    style.colors[StyleColor::WindowBg as usize] = [0.10, 0.10, 0.10, 1.00];
    style.colors[StyleColor::ChildBg as usize] = [0.00, 0.00, 0.00, 0.00];
    style.colors[StyleColor::PopupBg as usize] = [0.19, 0.19, 0.19, 0.92];
    style.colors[StyleColor::Border as usize] = [0.19, 0.19, 0.19, 0.29];
    style.colors[StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.24];
    style.colors[StyleColor::FrameBg as usize] = [0.05, 0.05, 0.05, 0.54];
    style.colors[StyleColor::FrameBgHovered as usize] = [0.19, 0.19, 0.19, 0.54];
    style.colors[StyleColor::FrameBgActive as usize] = [0.20, 0.22, 0.23, 1.00];
    style.colors[StyleColor::TitleBg as usize] = [0.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::TitleBgActive as usize] = [0.06, 0.06, 0.06, 1.00];
    style.colors[StyleColor::TitleBgCollapsed as usize] = [0.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::MenuBarBg as usize] = [0.14, 0.14, 0.14, 1.00];
    style.colors[StyleColor::ScrollbarBg as usize] = [0.05, 0.05, 0.05, 0.54];
    style.colors[StyleColor::ScrollbarGrab as usize] = [0.34, 0.34, 0.34, 0.54];
    style.colors[StyleColor::ScrollbarGrabHovered as usize] = [0.40, 0.40, 0.40, 0.54];
    style.colors[StyleColor::ScrollbarGrabActive as usize] = [0.56, 0.56, 0.56, 0.54];
    style.colors[StyleColor::CheckMark as usize] = [0.33, 0.67, 0.86, 1.00];
    style.colors[StyleColor::SliderGrab as usize] = [0.34, 0.34, 0.34, 0.54];
    style.colors[StyleColor::SliderGrabActive as usize] = [0.66, 0.56, 0.56, 0.54];
    style.colors[StyleColor::Button as usize] = [0.05, 0.05, 0.05, 0.54];
    style.colors[StyleColor::ButtonHovered as usize] = [0.19, 0.19, 0.19, 0.54];
    style.colors[StyleColor::ButtonActive as usize] = [0.20, 0.22, 0.23, 1.00];
    style.colors[StyleColor::Header as usize] = [0.00, 0.00, 0.00, 0.52];
    style.colors[StyleColor::HeaderHovered as usize] = [0.00, 0.00, 0.00, 0.36];
    style.colors[StyleColor::HeaderActive as usize] = [0.20, 0.22, 0.23, 0.33];
    style.colors[StyleColor::Separator as usize] = [0.28, 0.28, 0.28, 0.29];
    style.colors[StyleColor::SeparatorHovered as usize] = [0.44, 0.44, 0.44, 0.29];
    style.colors[StyleColor::SeparatorActive as usize] = [0.14, 0.44, 0.80, 1.00];
    style.colors[StyleColor::ResizeGrip as usize] = [0.28, 0.28, 0.28, 0.29];
    style.colors[StyleColor::ResizeGripHovered as usize] = [0.44, 0.44, 0.44, 0.29];
    style.colors[StyleColor::ResizeGripActive as usize] = [0.40, 0.44, 0.47, 1.00];
    style.colors[StyleColor::Tab as usize] = [0.00, 0.00, 0.00, 0.52];
    style.colors[StyleColor::TabHovered as usize] = [0.14, 0.14, 0.14, 1.00];
    style.colors[StyleColor::TabActive as usize] = [0.20, 0.20, 0.20, 0.36];
    style.colors[StyleColor::TabUnfocused as usize] = style.colors[StyleColor::Tab as usize];
    style.colors[StyleColor::TabUnfocusedActive as usize] =
        style.colors[StyleColor::TabHovered as usize];
    style.colors[StyleColor::DockingPreview as usize] = [0.33, 0.67, 0.86, 1.00];
    style.colors[StyleColor::DockingEmptyBg as usize] = [1.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::PlotLines as usize] = [1.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::PlotLinesHovered as usize] = [1.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::PlotHistogram as usize] = [1.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::PlotHistogramHovered as usize] = [1.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::TableHeaderBg as usize] = [0.00, 0.00, 0.00, 0.52];
    style.colors[StyleColor::TableBorderStrong as usize] = [0.00, 0.00, 0.00, 0.52];
    style.colors[StyleColor::TableBorderLight as usize] = [0.28, 0.28, 0.28, 0.29];
    style.colors[StyleColor::TableRowBg as usize] = [0.00, 0.00, 0.00, 0.00];
    style.colors[StyleColor::TableRowBgAlt as usize] = [1.00, 1.00, 1.00, 0.06];
    style.colors[StyleColor::TextSelectedBg as usize] = [0.20, 0.22, 0.23, 1.00];
    style.colors[StyleColor::DragDropTarget as usize] = [0.33, 0.67, 0.86, 1.00];
    style.colors[StyleColor::NavHighlight as usize] = [1.00, 0.00, 0.00, 1.00];
    style.colors[StyleColor::NavWindowingHighlight as usize] = [1.00, 0.00, 0.00, 0.70];
    style.colors[StyleColor::NavWindowingDimBg as usize] = [1.00, 0.00, 0.00, 0.20];
    style.colors[StyleColor::ModalWindowDimBg as usize] = [1.00, 0.00, 0.00, 0.35];
}
