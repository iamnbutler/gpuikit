use gpui::*;

pub fn h_stack() -> Div {
    div().flex()
}

pub fn v_stack() -> Div {
    div().flex().flex_col()
}

pub fn centered(child: impl IntoElement) -> Div {
    div().flex().items_center().justify_center().child(child)
}
