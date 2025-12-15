//! Default icon set based on Radix Icons
//!
//! Provides convenient access to the bundled Radix icon set.

use gpui::{svg, Svg};

/// Default icon set bundled with gpuikit (Radix Icons)
///
/// All icons are 15x15 SVGs designed to work well at small sizes.
///
/// # Example
/// ```ignore
/// use gpuikit::DefaultIcons;
///
/// let icon = DefaultIcons::star();
/// let button = icon_button("favorite", icon);
/// ```
pub struct Icons;

impl Icons {
    const PREFIX: &'static str = "icons/radix";

    fn icon(name: &str) -> Svg {
        svg().path(format!("{}/{}.svg", Self::PREFIX, name))
    }

    pub fn accessibility() -> Svg {
        Self::icon("accessibility")
    }
    pub fn activity_log() -> Svg {
        Self::icon("activity-log")
    }
    pub fn align_baseline() -> Svg {
        Self::icon("align-baseline")
    }
    pub fn align_bottom() -> Svg {
        Self::icon("align-bottom")
    }
    pub fn align_center_horizontally() -> Svg {
        Self::icon("align-center-horizontally")
    }
    pub fn align_center_vertically() -> Svg {
        Self::icon("align-center-vertically")
    }
    pub fn align_center() -> Svg {
        Self::icon("align-center")
    }
    pub fn align_end() -> Svg {
        Self::icon("align-end")
    }
    pub fn align_horizontal_centers() -> Svg {
        Self::icon("align-horizontal-centers")
    }
    pub fn align_left() -> Svg {
        Self::icon("align-left")
    }
    pub fn align_right() -> Svg {
        Self::icon("align-right")
    }
    pub fn align_start() -> Svg {
        Self::icon("align-start")
    }
    pub fn align_stretch() -> Svg {
        Self::icon("align-stretch")
    }
    pub fn align_top() -> Svg {
        Self::icon("align-top")
    }
    pub fn align_vertical_centers() -> Svg {
        Self::icon("align-vertical-centers")
    }
    pub fn all_sides() -> Svg {
        Self::icon("all-sides")
    }
    pub fn angle() -> Svg {
        Self::icon("angle")
    }
    pub fn archive() -> Svg {
        Self::icon("archive")
    }
    pub fn arrow_bottom_left() -> Svg {
        Self::icon("arrow-bottom-left")
    }
    pub fn arrow_bottom_right() -> Svg {
        Self::icon("arrow-bottom-right")
    }
    pub fn arrow_down() -> Svg {
        Self::icon("arrow-down")
    }
    pub fn arrow_left() -> Svg {
        Self::icon("arrow-left")
    }
    pub fn arrow_right() -> Svg {
        Self::icon("arrow-right")
    }
    pub fn arrow_top_left() -> Svg {
        Self::icon("arrow-top-left")
    }
    pub fn arrow_top_right() -> Svg {
        Self::icon("arrow-top-right")
    }
    pub fn arrow_up() -> Svg {
        Self::icon("arrow-up")
    }
    pub fn aspect_ratio() -> Svg {
        Self::icon("aspect-ratio")
    }
    pub fn avatar() -> Svg {
        Self::icon("avatar")
    }
    pub fn backpack() -> Svg {
        Self::icon("backpack")
    }
    pub fn badge() -> Svg {
        Self::icon("badge")
    }
    pub fn bar_chart() -> Svg {
        Self::icon("bar-chart")
    }
    pub fn bell() -> Svg {
        Self::icon("bell")
    }
    pub fn blending_mode() -> Svg {
        Self::icon("blending-mode")
    }
    pub fn bookmark_filled() -> Svg {
        Self::icon("bookmark-filled")
    }
    pub fn bookmark() -> Svg {
        Self::icon("bookmark")
    }
    pub fn border_all() -> Svg {
        Self::icon("border-all")
    }
    pub fn border_bottom() -> Svg {
        Self::icon("border-bottom")
    }
    pub fn border_dashed() -> Svg {
        Self::icon("border-dashed")
    }
    pub fn border_dotted() -> Svg {
        Self::icon("border-dotted")
    }
    pub fn border_left() -> Svg {
        Self::icon("border-left")
    }
    pub fn border_none() -> Svg {
        Self::icon("border-none")
    }
    pub fn border_right() -> Svg {
        Self::icon("border-right")
    }
    pub fn border_solid() -> Svg {
        Self::icon("border-solid")
    }
    pub fn border_split() -> Svg {
        Self::icon("border-split")
    }
    pub fn border_style() -> Svg {
        Self::icon("border-style")
    }
    pub fn border_top() -> Svg {
        Self::icon("border-top")
    }
    pub fn border_width() -> Svg {
        Self::icon("border-width")
    }
    pub fn box_model() -> Svg {
        Self::icon("box-model")
    }
    pub fn box_() -> Svg {
        Self::icon("box")
    }
    pub fn button() -> Svg {
        Self::icon("button")
    }
    pub fn calendar() -> Svg {
        Self::icon("calendar")
    }
    pub fn camera() -> Svg {
        Self::icon("camera")
    }
    pub fn card_stack_minus() -> Svg {
        Self::icon("card-stack-minus")
    }
    pub fn card_stack_plus() -> Svg {
        Self::icon("card-stack-plus")
    }
    pub fn card_stack() -> Svg {
        Self::icon("card-stack")
    }
    pub fn caret_down() -> Svg {
        Self::icon("caret-down")
    }
    pub fn caret_left() -> Svg {
        Self::icon("caret-left")
    }
    pub fn caret_right() -> Svg {
        Self::icon("caret-right")
    }
    pub fn caret_sort() -> Svg {
        Self::icon("caret-sort")
    }
    pub fn caret_up() -> Svg {
        Self::icon("caret-up")
    }
    pub fn chat_bubble() -> Svg {
        Self::icon("chat-bubble")
    }
    pub fn check_circled() -> Svg {
        Self::icon("check-circled")
    }
    pub fn check() -> Svg {
        Self::icon("check")
    }
    pub fn checkbox() -> Svg {
        Self::icon("checkbox")
    }
    pub fn chevron_down() -> Svg {
        Self::icon("chevron-down")
    }
    pub fn chevron_left() -> Svg {
        Self::icon("chevron-left")
    }
    pub fn chevron_right() -> Svg {
        Self::icon("chevron-right")
    }
    pub fn chevron_up() -> Svg {
        Self::icon("chevron-up")
    }
    pub fn circle_backslash() -> Svg {
        Self::icon("circle-backslash")
    }
    pub fn circle() -> Svg {
        Self::icon("circle")
    }
    pub fn clipboard_copy() -> Svg {
        Self::icon("clipboard-copy")
    }
    pub fn clipboard() -> Svg {
        Self::icon("clipboard")
    }
    pub fn clock() -> Svg {
        Self::icon("clock")
    }
    pub fn code() -> Svg {
        Self::icon("code")
    }
    pub fn codesandbox_logo() -> Svg {
        Self::icon("codesandbox-logo")
    }
    pub fn color_wheel() -> Svg {
        Self::icon("color-wheel")
    }
    pub fn column_spacing() -> Svg {
        Self::icon("column-spacing")
    }
    pub fn columns() -> Svg {
        Self::icon("columns")
    }
    pub fn commit() -> Svg {
        Self::icon("commit")
    }
    pub fn component_1() -> Svg {
        Self::icon("component-1")
    }
    pub fn component_2() -> Svg {
        Self::icon("component-2")
    }
    pub fn component_boolean() -> Svg {
        Self::icon("component-boolean")
    }
    pub fn component_instance() -> Svg {
        Self::icon("component-instance")
    }
    pub fn component_none() -> Svg {
        Self::icon("component-none")
    }
    pub fn component_placeholder() -> Svg {
        Self::icon("component-placeholder")
    }
    pub fn container() -> Svg {
        Self::icon("container")
    }
    pub fn cookie() -> Svg {
        Self::icon("cookie")
    }
    pub fn copy() -> Svg {
        Self::icon("copy")
    }
    pub fn corner_bottom_left() -> Svg {
        Self::icon("corner-bottom-left")
    }
    pub fn corner_bottom_right() -> Svg {
        Self::icon("corner-bottom-right")
    }
    pub fn corner_top_left() -> Svg {
        Self::icon("corner-top-left")
    }
    pub fn corner_top_right() -> Svg {
        Self::icon("corner-top-right")
    }
    pub fn corners() -> Svg {
        Self::icon("corners")
    }
    pub fn countdown_timer() -> Svg {
        Self::icon("countdown-timer")
    }
    pub fn counter_clockwise_clock() -> Svg {
        Self::icon("counter-clockwise-clock")
    }
    pub fn crop() -> Svg {
        Self::icon("crop")
    }
    pub fn cross_1() -> Svg {
        Self::icon("cross-1")
    }
    pub fn cross_2() -> Svg {
        Self::icon("cross-2")
    }
    pub fn cross_circled() -> Svg {
        Self::icon("cross-circled")
    }
    pub fn crosshair_1() -> Svg {
        Self::icon("crosshair-1")
    }
    pub fn crosshair_2() -> Svg {
        Self::icon("crosshair-2")
    }
    pub fn crumpled_paper() -> Svg {
        Self::icon("crumpled-paper")
    }
    pub fn cube() -> Svg {
        Self::icon("cube")
    }
    pub fn cursor_arrow() -> Svg {
        Self::icon("cursor-arrow")
    }
    pub fn cursor_text() -> Svg {
        Self::icon("cursor-text")
    }
    pub fn dash() -> Svg {
        Self::icon("dash")
    }
    pub fn dashboard() -> Svg {
        Self::icon("dashboard")
    }
    pub fn database() -> Svg {
        Self::icon("database")
    }
    pub fn desktop() -> Svg {
        Self::icon("desktop")
    }
    pub fn dimensions() -> Svg {
        Self::icon("dimensions")
    }
    pub fn disc() -> Svg {
        Self::icon("disc")
    }
    pub fn discord_logo() -> Svg {
        Self::icon("discord-logo")
    }
    pub fn divider_horizontal() -> Svg {
        Self::icon("divider-horizontal")
    }
    pub fn divider_vertical() -> Svg {
        Self::icon("divider-vertical")
    }
    pub fn dot_filled() -> Svg {
        Self::icon("dot-filled")
    }
    pub fn dot_solid() -> Svg {
        Self::icon("dot-solid")
    }
    pub fn dot() -> Svg {
        Self::icon("dot")
    }
    pub fn dots_horizontal() -> Svg {
        Self::icon("dots-horizontal")
    }
    pub fn dots_vertical() -> Svg {
        Self::icon("dots-vertical")
    }
    pub fn double_arrow_down() -> Svg {
        Self::icon("double-arrow-down")
    }
    pub fn double_arrow_left() -> Svg {
        Self::icon("double-arrow-left")
    }
    pub fn double_arrow_right() -> Svg {
        Self::icon("double-arrow-right")
    }
    pub fn double_arrow_up() -> Svg {
        Self::icon("double-arrow-up")
    }
    pub fn download() -> Svg {
        Self::icon("download")
    }
    pub fn drag_handle_dots_1() -> Svg {
        Self::icon("drag-handle-dots-1")
    }
    pub fn drag_handle_dots_2() -> Svg {
        Self::icon("drag-handle-dots-2")
    }
    pub fn drag_handle_horizontal() -> Svg {
        Self::icon("drag-handle-horizontal")
    }
    pub fn drag_handle_vertical() -> Svg {
        Self::icon("drag-handle-vertical")
    }
    pub fn drawing_pin_filled() -> Svg {
        Self::icon("drawing-pin-filled")
    }
    pub fn drawing_pin_solid() -> Svg {
        Self::icon("drawing-pin-solid")
    }
    pub fn drawing_pin() -> Svg {
        Self::icon("drawing-pin")
    }
    pub fn dropdown_menu() -> Svg {
        Self::icon("dropdown-menu")
    }
    pub fn enter_full_screen() -> Svg {
        Self::icon("enter-full-screen")
    }
    pub fn enter() -> Svg {
        Self::icon("enter")
    }
    pub fn envelope_closed() -> Svg {
        Self::icon("envelope-closed")
    }
    pub fn envelope_open() -> Svg {
        Self::icon("envelope-open")
    }
    pub fn eraser() -> Svg {
        Self::icon("eraser")
    }
    pub fn exclamation_circled() -> Svg {
        Self::icon("exclamation-circled")
    }
    pub fn exclamation_mark() -> Svg {
        Self::icon("exclamation-mark")
    }
    pub fn exclamation_triangle() -> Svg {
        Self::icon("exclamation-triangle")
    }
    pub fn exit_full_screen() -> Svg {
        Self::icon("exit-full-screen")
    }
    pub fn exit() -> Svg {
        Self::icon("exit")
    }
    pub fn external_link() -> Svg {
        Self::icon("external-link")
    }
    pub fn eye_closed() -> Svg {
        Self::icon("eye-closed")
    }
    pub fn eye_none() -> Svg {
        Self::icon("eye-none")
    }
    pub fn eye_open() -> Svg {
        Self::icon("eye-open")
    }
    pub fn face() -> Svg {
        Self::icon("face")
    }
    pub fn figma_logo() -> Svg {
        Self::icon("figma-logo")
    }
    pub fn file_minus() -> Svg {
        Self::icon("file-minus")
    }
    pub fn file_plus() -> Svg {
        Self::icon("file-plus")
    }
    pub fn file_text() -> Svg {
        Self::icon("file-text")
    }
    pub fn file() -> Svg {
        Self::icon("file")
    }
    pub fn filter() -> Svg {
        Self::icon("filter")
    }
    pub fn font_bold() -> Svg {
        Self::icon("font-bold")
    }
    pub fn font_family() -> Svg {
        Self::icon("font-family")
    }
    pub fn font_italic() -> Svg {
        Self::icon("font-italic")
    }
    pub fn font_roman() -> Svg {
        Self::icon("font-roman")
    }
    pub fn font_size() -> Svg {
        Self::icon("font-size")
    }
    pub fn font_style() -> Svg {
        Self::icon("font-style")
    }
    pub fn frame() -> Svg {
        Self::icon("frame")
    }
    pub fn framer_logo() -> Svg {
        Self::icon("framer-logo")
    }
    pub fn gear() -> Svg {
        Self::icon("gear")
    }
    pub fn github_logo() -> Svg {
        Self::icon("github-logo")
    }
    pub fn globe_2() -> Svg {
        Self::icon("globe-2")
    }
    pub fn globe() -> Svg {
        Self::icon("globe")
    }
    pub fn grid() -> Svg {
        Self::icon("grid")
    }
    pub fn group() -> Svg {
        Self::icon("group")
    }
    pub fn half_1() -> Svg {
        Self::icon("half-1")
    }
    pub fn half_2() -> Svg {
        Self::icon("half-2")
    }
    pub fn hamburger_menu() -> Svg {
        Self::icon("hamburger-menu")
    }
    pub fn hand() -> Svg {
        Self::icon("hand")
    }
    pub fn heading() -> Svg {
        Self::icon("heading")
    }
    pub fn heart_filled() -> Svg {
        Self::icon("heart-filled")
    }
    pub fn heart() -> Svg {
        Self::icon("heart")
    }
    pub fn height() -> Svg {
        Self::icon("height")
    }
    pub fn hobby_knife() -> Svg {
        Self::icon("hobby-knife")
    }
    pub fn home() -> Svg {
        Self::icon("home")
    }
    pub fn iconjar_logo() -> Svg {
        Self::icon("iconjar-logo")
    }
    pub fn id_card() -> Svg {
        Self::icon("id-card")
    }
    pub fn image() -> Svg {
        Self::icon("image")
    }
    pub fn info_circled() -> Svg {
        Self::icon("info-circled")
    }
    pub fn inner_shadow() -> Svg {
        Self::icon("inner-shadow")
    }
    pub fn input() -> Svg {
        Self::icon("input")
    }
    pub fn instagram_logo() -> Svg {
        Self::icon("instagram-logo")
    }
    pub fn justify_center() -> Svg {
        Self::icon("justify-center")
    }
    pub fn justify_end() -> Svg {
        Self::icon("justify-end")
    }
    pub fn justify_start() -> Svg {
        Self::icon("justify-start")
    }
    pub fn justify_stretch() -> Svg {
        Self::icon("justify-stretch")
    }
    pub fn keyboard() -> Svg {
        Self::icon("keyboard")
    }
    pub fn lap_timer() -> Svg {
        Self::icon("lap-timer")
    }
    pub fn laptop() -> Svg {
        Self::icon("laptop")
    }
    pub fn layers() -> Svg {
        Self::icon("layers")
    }
    pub fn layout() -> Svg {
        Self::icon("layout")
    }
    pub fn letter_case_capitalize() -> Svg {
        Self::icon("letter-case-capitalize")
    }
    pub fn letter_case_lowercase() -> Svg {
        Self::icon("letter-case-lowercase")
    }
    pub fn letter_case_toggle() -> Svg {
        Self::icon("letter-case-toggle")
    }
    pub fn letter_case_uppercase() -> Svg {
        Self::icon("letter-case-uppercase")
    }
    pub fn letter_spacing() -> Svg {
        Self::icon("letter-spacing")
    }
    pub fn lightning_bolt() -> Svg {
        Self::icon("lightning-bolt")
    }
    pub fn line_height() -> Svg {
        Self::icon("line-height")
    }
    pub fn link_1() -> Svg {
        Self::icon("link-1")
    }
    pub fn link_2() -> Svg {
        Self::icon("link-2")
    }
    pub fn link_break_1() -> Svg {
        Self::icon("link-break-1")
    }
    pub fn link_break_2() -> Svg {
        Self::icon("link-break-2")
    }
    pub fn link_none_1() -> Svg {
        Self::icon("link-none-1")
    }
    pub fn link_none_2() -> Svg {
        Self::icon("link-none-2")
    }
    pub fn linkedin_logo() -> Svg {
        Self::icon("linkedin-logo")
    }
    pub fn list_bullet() -> Svg {
        Self::icon("list-bullet")
    }
    pub fn lock_closed() -> Svg {
        Self::icon("lock-closed")
    }
    pub fn lock_open_1() -> Svg {
        Self::icon("lock-open-1")
    }
    pub fn lock_open_2() -> Svg {
        Self::icon("lock-open-2")
    }
    pub fn loop_() -> Svg {
        Self::icon("loop")
    }
    pub fn magic_wand() -> Svg {
        Self::icon("magic-wand")
    }
    pub fn magnifying_glass() -> Svg {
        Self::icon("magnifying-glass")
    }
    pub fn margin() -> Svg {
        Self::icon("margin")
    }
    pub fn mask_off() -> Svg {
        Self::icon("mask-off")
    }
    pub fn mask_on() -> Svg {
        Self::icon("mask-on")
    }
    pub fn minimize() -> Svg {
        Self::icon("minimize")
    }
    pub fn minus_circled() -> Svg {
        Self::icon("minus-circled")
    }
    pub fn minus() -> Svg {
        Self::icon("minus")
    }
    pub fn mix() -> Svg {
        Self::icon("mix")
    }
    pub fn mixer_horizontal() -> Svg {
        Self::icon("mixer-horizontal")
    }
    pub fn mixer_vertical() -> Svg {
        Self::icon("mixer-vertical")
    }
    pub fn mobile() -> Svg {
        Self::icon("mobile")
    }
    pub fn modulz_logo() -> Svg {
        Self::icon("modulz-logo")
    }
    pub fn moon() -> Svg {
        Self::icon("moon")
    }
    pub fn move_() -> Svg {
        Self::icon("move")
    }
    pub fn notion_logo() -> Svg {
        Self::icon("notion-logo")
    }
    pub fn opacity() -> Svg {
        Self::icon("opacity")
    }
    pub fn open_in_new_window() -> Svg {
        Self::icon("open-in-new-window")
    }
    pub fn outer_shadow() -> Svg {
        Self::icon("outer-shadow")
    }
    pub fn overline() -> Svg {
        Self::icon("overline")
    }
    pub fn padding() -> Svg {
        Self::icon("padding")
    }
    pub fn panel_bottom_minimized() -> Svg {
        Self::icon("panel-bottom-minimized")
    }
    pub fn panel_bottom() -> Svg {
        Self::icon("panel-bottom")
    }
    pub fn panel_left_minimized() -> Svg {
        Self::icon("panel-left-minimized")
    }
    pub fn panel_left() -> Svg {
        Self::icon("panel-left")
    }
    pub fn panel_right_minimized() -> Svg {
        Self::icon("panel-right-minimized")
    }
    pub fn panel_right() -> Svg {
        Self::icon("panel-right")
    }
    pub fn paper_plane() -> Svg {
        Self::icon("paper-plane")
    }
    pub fn pause() -> Svg {
        Self::icon("pause")
    }
    pub fn pencil_1() -> Svg {
        Self::icon("pencil-1")
    }
    pub fn pencil_2() -> Svg {
        Self::icon("pencil-2")
    }
    pub fn people() -> Svg {
        Self::icon("people")
    }
    pub fn person() -> Svg {
        Self::icon("person")
    }
    pub fn pie_chart() -> Svg {
        Self::icon("pie-chart")
    }
    pub fn pilcrow() -> Svg {
        Self::icon("pilcrow")
    }
    pub fn pin_bottom() -> Svg {
        Self::icon("pin-bottom")
    }
    pub fn pin_left() -> Svg {
        Self::icon("pin-left")
    }
    pub fn pin_right() -> Svg {
        Self::icon("pin-right")
    }
    pub fn pin_top() -> Svg {
        Self::icon("pin-top")
    }
    pub fn play() -> Svg {
        Self::icon("play")
    }
    pub fn plus_circled() -> Svg {
        Self::icon("plus-circled")
    }
    pub fn plus() -> Svg {
        Self::icon("plus")
    }
    pub fn question_mark_circled() -> Svg {
        Self::icon("question-mark-circled")
    }
    pub fn question_mark() -> Svg {
        Self::icon("question-mark")
    }
    pub fn quote() -> Svg {
        Self::icon("quote")
    }
    pub fn radiobutton() -> Svg {
        Self::icon("radiobutton")
    }
    pub fn reader() -> Svg {
        Self::icon("reader")
    }
    pub fn reload() -> Svg {
        Self::icon("reload")
    }
    pub fn reset() -> Svg {
        Self::icon("reset")
    }
    pub fn resume() -> Svg {
        Self::icon("resume")
    }
    pub fn rocket() -> Svg {
        Self::icon("rocket")
    }
    pub fn rotate_counter_clockwise() -> Svg {
        Self::icon("rotate-counter-clockwise")
    }
    pub fn row_spacing() -> Svg {
        Self::icon("row-spacing")
    }
    pub fn rows() -> Svg {
        Self::icon("rows")
    }
    pub fn ruler_horizontal() -> Svg {
        Self::icon("ruler-horizontal")
    }
    pub fn ruler_square() -> Svg {
        Self::icon("ruler-square")
    }
    pub fn scissors() -> Svg {
        Self::icon("scissors")
    }
    pub fn section() -> Svg {
        Self::icon("section")
    }
    pub fn server() -> Svg {
        Self::icon("server")
    }
    pub fn sewing_pin_filled() -> Svg {
        Self::icon("sewing-pin-filled")
    }
    pub fn sewing_pin_solid() -> Svg {
        Self::icon("sewing-pin-solid")
    }
    pub fn sewing_pin() -> Svg {
        Self::icon("sewing-pin")
    }
    pub fn shadow_inner() -> Svg {
        Self::icon("shadow-inner")
    }
    pub fn shadow_none() -> Svg {
        Self::icon("shadow-none")
    }
    pub fn shadow_outer() -> Svg {
        Self::icon("shadow-outer")
    }
    pub fn shadow() -> Svg {
        Self::icon("shadow")
    }
    pub fn share_1() -> Svg {
        Self::icon("share-1")
    }
    pub fn share_2() -> Svg {
        Self::icon("share-2")
    }
    pub fn shuffle() -> Svg {
        Self::icon("shuffle")
    }
    pub fn size() -> Svg {
        Self::icon("size")
    }
    pub fn sketch_logo() -> Svg {
        Self::icon("sketch-logo")
    }
    pub fn slash() -> Svg {
        Self::icon("slash")
    }
    pub fn slider() -> Svg {
        Self::icon("slider")
    }
    pub fn space_between_horizontally() -> Svg {
        Self::icon("space-between-horizontally")
    }
    pub fn space_between_vertically() -> Svg {
        Self::icon("space-between-vertically")
    }
    pub fn space_evenly_horizontally() -> Svg {
        Self::icon("space-evenly-horizontally")
    }
    pub fn space_evenly_vertically() -> Svg {
        Self::icon("space-evenly-vertically")
    }
    pub fn speaker_loud() -> Svg {
        Self::icon("speaker-loud")
    }
    pub fn speaker_moderate() -> Svg {
        Self::icon("speaker-moderate")
    }
    pub fn speaker_off() -> Svg {
        Self::icon("speaker-off")
    }
    pub fn speaker_quiet() -> Svg {
        Self::icon("speaker-quiet")
    }
    pub fn square() -> Svg {
        Self::icon("square")
    }
    pub fn stack() -> Svg {
        Self::icon("stack")
    }
    pub fn star_filled() -> Svg {
        Self::icon("star-filled")
    }
    pub fn star() -> Svg {
        Self::icon("star")
    }
    pub fn stitches_logo() -> Svg {
        Self::icon("stitches-logo")
    }
    pub fn stop() -> Svg {
        Self::icon("stop")
    }
    pub fn stopwatch() -> Svg {
        Self::icon("stopwatch")
    }
    pub fn stretch_horizontally() -> Svg {
        Self::icon("stretch-horizontally")
    }
    pub fn stretch_vertically() -> Svg {
        Self::icon("stretch-vertically")
    }
    pub fn strikethrough() -> Svg {
        Self::icon("strikethrough")
    }
    pub fn sun() -> Svg {
        Self::icon("sun")
    }
    pub fn switch() -> Svg {
        Self::icon("switch")
    }
    pub fn symbol() -> Svg {
        Self::icon("symbol")
    }
    pub fn table() -> Svg {
        Self::icon("table")
    }
    pub fn target() -> Svg {
        Self::icon("target")
    }
    pub fn text_align_bottom() -> Svg {
        Self::icon("text-align-bottom")
    }
    pub fn text_align_center() -> Svg {
        Self::icon("text-align-center")
    }
    pub fn text_align_justify() -> Svg {
        Self::icon("text-align-justify")
    }
    pub fn text_align_left() -> Svg {
        Self::icon("text-align-left")
    }
    pub fn text_align_middle() -> Svg {
        Self::icon("text-align-middle")
    }
    pub fn text_align_right() -> Svg {
        Self::icon("text-align-right")
    }
    pub fn text_align_top() -> Svg {
        Self::icon("text-align-top")
    }
    pub fn text_none() -> Svg {
        Self::icon("text-none")
    }
    pub fn text() -> Svg {
        Self::icon("text")
    }
    pub fn thick_arrow_down() -> Svg {
        Self::icon("thick-arrow-down")
    }
    pub fn thick_arrow_left() -> Svg {
        Self::icon("thick-arrow-left")
    }
    pub fn thick_arrow_right() -> Svg {
        Self::icon("thick-arrow-right")
    }
    pub fn thick_arrow_up() -> Svg {
        Self::icon("thick-arrow-up")
    }
    pub fn timer() -> Svg {
        Self::icon("timer")
    }
    pub fn tokens() -> Svg {
        Self::icon("tokens")
    }
    pub fn track_next() -> Svg {
        Self::icon("track-next")
    }
    pub fn track_previous() -> Svg {
        Self::icon("track-previous")
    }
    pub fn transform() -> Svg {
        Self::icon("transform")
    }
    pub fn transparency_grid() -> Svg {
        Self::icon("transparency-grid")
    }
    pub fn trash() -> Svg {
        Self::icon("trash")
    }
    pub fn triangle_down() -> Svg {
        Self::icon("triangle-down")
    }
    pub fn triangle_left() -> Svg {
        Self::icon("triangle-left")
    }
    pub fn triangle_right() -> Svg {
        Self::icon("triangle-right")
    }
    pub fn triangle_up() -> Svg {
        Self::icon("triangle-up")
    }
    pub fn twitter_logo() -> Svg {
        Self::icon("twitter-logo")
    }
    pub fn underline() -> Svg {
        Self::icon("underline")
    }
    pub fn update() -> Svg {
        Self::icon("update")
    }
    pub fn upload() -> Svg {
        Self::icon("upload")
    }
    pub fn value_none() -> Svg {
        Self::icon("value-none")
    }
    pub fn value() -> Svg {
        Self::icon("value")
    }
    pub fn vercel_logo() -> Svg {
        Self::icon("vercel-logo")
    }
    pub fn video() -> Svg {
        Self::icon("video")
    }
    pub fn view_grid() -> Svg {
        Self::icon("view-grid")
    }
    pub fn view_horizontal() -> Svg {
        Self::icon("view-horizontal")
    }
    pub fn view_none() -> Svg {
        Self::icon("view-none")
    }
    pub fn view_vertical() -> Svg {
        Self::icon("view-vertical")
    }
    pub fn width() -> Svg {
        Self::icon("width")
    }
    pub fn zoom_in() -> Svg {
        Self::icon("zoom-in")
    }
    pub fn zoom_out() -> Svg {
        Self::icon("zoom-out")
    }
}
