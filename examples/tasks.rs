#![allow(missing_docs)]

use gpui::{
    div, hsla, prelude::FluentBuilder, px, size, App, AppContext, Application, Bounds, ClickEvent,
    Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement,
    Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use gpuikit::{
    elements::{
        badge::{badge, BadgeVariant},
        button::button,
    },
    layout::{h_stack, v_stack},
    theme::{ActiveTheme, Themeable},
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum TaskStatus {
    Todo,
    InProgress,
    Backlog,
    Done,
    Canceled,
}

impl TaskStatus {
    fn label(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "Todo",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Backlog => "Backlog",
            TaskStatus::Done => "Done",
            TaskStatus::Canceled => "Canceled",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "○",
            TaskStatus::InProgress => "◔",
            TaskStatus::Backlog => "◷",
            TaskStatus::Done => "●",
            TaskStatus::Canceled => "⊘",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TaskPriority {
    Low,
    Medium,
    High,
}

impl TaskPriority {
    fn label(&self) -> &'static str {
        match self {
            TaskPriority::Low => "Low",
            TaskPriority::Medium => "Medium",
            TaskPriority::High => "High",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TaskLabel {
    Bug,
    Feature,
    Documentation,
}

impl TaskLabel {
    fn label(&self) -> &'static str {
        match self {
            TaskLabel::Bug => "Bug",
            TaskLabel::Feature => "Feature",
            TaskLabel::Documentation => "Docs",
        }
    }
}

#[derive(Clone)]
struct Task {
    id: String,
    title: String,
    label: TaskLabel,
    status: TaskStatus,
    priority: TaskPriority,
}

fn generate_tasks() -> Vec<Task> {
    let tasks_data = [
        (
            "AUTH-101",
            "Implement OAuth2 login flow",
            TaskLabel::Feature,
            TaskStatus::InProgress,
            TaskPriority::High,
        ),
        (
            "AUTH-102",
            "Fix session timeout handling",
            TaskLabel::Bug,
            TaskStatus::Todo,
            TaskPriority::High,
        ),
        (
            "DB-201",
            "Add database connection pooling",
            TaskLabel::Feature,
            TaskStatus::Done,
            TaskPriority::Medium,
        ),
        (
            "DB-202",
            "Fix N+1 query in user list",
            TaskLabel::Bug,
            TaskStatus::InProgress,
            TaskPriority::High,
        ),
        (
            "API-301",
            "Document REST API endpoints",
            TaskLabel::Documentation,
            TaskStatus::Backlog,
            TaskPriority::Low,
        ),
        (
            "API-302",
            "Add rate limiting middleware",
            TaskLabel::Feature,
            TaskStatus::Todo,
            TaskPriority::Medium,
        ),
        (
            "UI-401",
            "Update button component styles",
            TaskLabel::Feature,
            TaskStatus::Done,
            TaskPriority::Low,
        ),
        (
            "UI-402",
            "Fix modal z-index issue",
            TaskLabel::Bug,
            TaskStatus::Canceled,
            TaskPriority::Medium,
        ),
        (
            "UI-403",
            "Add dark mode support",
            TaskLabel::Feature,
            TaskStatus::InProgress,
            TaskPriority::Medium,
        ),
        (
            "PERF-501",
            "Optimize image loading",
            TaskLabel::Feature,
            TaskStatus::Backlog,
            TaskPriority::Medium,
        ),
        (
            "PERF-502",
            "Fix memory leak in cache",
            TaskLabel::Bug,
            TaskStatus::Todo,
            TaskPriority::High,
        ),
        (
            "PERF-503",
            "Add lazy loading for lists",
            TaskLabel::Feature,
            TaskStatus::Done,
            TaskPriority::Low,
        ),
        (
            "SEC-601",
            "Update security documentation",
            TaskLabel::Documentation,
            TaskStatus::Done,
            TaskPriority::Medium,
        ),
        (
            "SEC-602",
            "Fix XSS vulnerability",
            TaskLabel::Bug,
            TaskStatus::InProgress,
            TaskPriority::High,
        ),
        (
            "SEC-603",
            "Add input sanitization",
            TaskLabel::Feature,
            TaskStatus::Todo,
            TaskPriority::High,
        ),
        (
            "TEST-701",
            "Write unit tests for auth",
            TaskLabel::Documentation,
            TaskStatus::Backlog,
            TaskPriority::Medium,
        ),
        (
            "TEST-702",
            "Add integration tests",
            TaskLabel::Feature,
            TaskStatus::Todo,
            TaskPriority::Low,
        ),
        (
            "TEST-703",
            "Fix flaky CI tests",
            TaskLabel::Bug,
            TaskStatus::InProgress,
            TaskPriority::Medium,
        ),
        (
            "INFRA-801",
            "Update deployment docs",
            TaskLabel::Documentation,
            TaskStatus::Done,
            TaskPriority::Low,
        ),
        (
            "INFRA-802",
            "Configure auto-scaling",
            TaskLabel::Feature,
            TaskStatus::Backlog,
            TaskPriority::Medium,
        ),
    ];

    tasks_data
        .iter()
        .map(|(id, title, label, status, priority)| Task {
            id: id.to_string(),
            title: title.to_string(),
            label: *label,
            status: *status,
            priority: *priority,
        })
        .collect()
}

struct RowCheckbox {
    checked: bool,
    task_index: usize,
}

struct RowCheckboxChanged {
    #[allow(dead_code)]
    task_index: usize,
    checked: bool,
}

impl EventEmitter<RowCheckboxChanged> for RowCheckbox {}

impl RowCheckbox {
    fn new(task_index: usize) -> Self {
        Self {
            checked: false,
            task_index,
        }
    }

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.checked = !self.checked;
        cx.emit(RowCheckboxChanged {
            task_index: self.task_index,
            checked: self.checked,
        });
        cx.notify();
    }
}

impl Render for RowCheckbox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let checked = self.checked;

        div()
            .id(ElementId::NamedInteger(
                "row-checkbox".into(),
                self.task_index as u64,
            ))
            .size(px(14.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .bg(if checked {
                theme.accent()
            } else {
                theme.surface()
            })
            .border_1()
            .border_color(if checked {
                theme.accent()
            } else {
                theme.border()
            })
            .rounded(px(2.))
            .cursor_pointer()
            .hover(|s| s.border_color(theme.border_secondary()))
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .on_click(cx.listener(|this, _, _, cx| {
                this.toggle(cx);
            }))
            .when(checked, |this| {
                this.child(div().text_xs().text_color(theme.surface()).child("✓"))
            })
    }
}

struct TasksApp {
    focus_handle: FocusHandle,
    tasks: Vec<Task>,
    row_checkboxes: Vec<Entity<RowCheckbox>>,
    selected_count: usize,
    current_page: usize,
    rows_per_page: usize,
    _subscriptions: Vec<gpui::Subscription>,
}

impl TasksApp {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let tasks = generate_tasks();
        let mut subscriptions = Vec::new();

        let row_checkboxes: Vec<_> = tasks
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let checkbox = cx.new(|_| RowCheckbox::new(i));
                subscriptions.push(cx.subscribe(&checkbox, |this, _, event, cx| {
                    this.on_row_checkbox_changed(event, cx);
                }));
                checkbox
            })
            .collect();

        Self {
            focus_handle: cx.focus_handle(),
            tasks,
            row_checkboxes,
            selected_count: 0,
            current_page: 0,
            rows_per_page: 10,
            _subscriptions: subscriptions,
        }
    }

    fn on_row_checkbox_changed(&mut self, event: &RowCheckboxChanged, cx: &mut Context<Self>) {
        if event.checked {
            self.selected_count += 1;
        } else {
            self.selected_count = self.selected_count.saturating_sub(1);
        }
        cx.notify();
    }

    fn total_pages(&self) -> usize {
        (self.tasks.len() + self.rows_per_page - 1) / self.rows_per_page
    }

    fn visible_tasks(&self) -> impl Iterator<Item = (usize, &Task)> {
        let start = self.current_page * self.rows_per_page;
        let end = (start + self.rows_per_page).min(self.tasks.len());
        self.tasks[start..end]
            .iter()
            .enumerate()
            .map(move |(i, t)| (start + i, t))
    }

    fn go_to_page(&mut self, page: usize, cx: &mut Context<Self>) {
        let max_page = self.total_pages().saturating_sub(1);
        self.current_page = page.min(max_page);
        cx.notify();
    }

    fn next_page(&mut self, cx: &mut Context<Self>) {
        self.go_to_page(self.current_page + 1, cx);
    }

    fn prev_page(&mut self, cx: &mut Context<Self>) {
        self.go_to_page(self.current_page.saturating_sub(1), cx);
    }

    fn first_page(&mut self, cx: &mut Context<Self>) {
        self.go_to_page(0, cx);
    }

    fn last_page(&mut self, cx: &mut Context<Self>) {
        self.go_to_page(self.total_pages().saturating_sub(1), cx);
    }
}

impl Focusable for TasksApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TasksApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let visible_tasks: Vec<_> = self.visible_tasks().collect();
        let total_pages = self.total_pages();
        let current_page = self.current_page;
        let total_tasks = self.tasks.len();
        let selected_count = self.selected_count;

        div()
            .id("tasks-app")
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.bg())
            .text_color(theme.fg())
            .child(
                v_stack()
                    .id("list")
                    .flex_1()
                    .p(px(16.))
                    .gap(px(8.))
                    .overflow_y_scroll()
                    .child(
                        h_stack()
                            .justify_between()
                            .items_center()
                            .child(
                                h_stack()
                                    .gap(px(6.))
                                    .child(
                                        div()
                                            .px(px(8.))
                                            .py(px(4.))
                                            .min_w(px(160.))
                                            .bg(theme.surface())
                                            .border_1()
                                            .border_color(theme.border())
                                            .rounded(px(4.))
                                            .text_xs()
                                            .text_color(theme.fg_muted())
                                            .child("Filter..."),
                                    )
                                    .child(filter_button("Status", &theme))
                                    .child(filter_button("Priority", &theme)),
                            )
                            .child(
                                h_stack()
                                    .gap(px(6.))
                                    .child(button("view-btn", "View"))
                                    .child(
                                        button("add-task-btn", "New Task").on_click(|_, _, _| {}),
                                    ),
                            ),
                    )
                    .child(
                        v_stack()
                            .border_1()
                            .border_color(theme.border())
                            .rounded(px(4.))
                            .overflow_hidden()
                            .child(
                                h_stack()
                                    .w_full()
                                    .px(px(8.))
                                    .py(px(6.))
                                    .bg(theme.surface())
                                    .border_b_1()
                                    .border_color(theme.border())
                                    .items_center()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child(div().w(px(28.)).flex_none())
                                    .child(div().w(px(80.)).flex_none().child("Task"))
                                    .child(div().flex_1().child("Title"))
                                    .child(div().w(px(90.)).flex_none().child("Status"))
                                    .child(div().w(px(70.)).flex_none().child("Priority"))
                                    .child(div().w(px(28.)).flex_none()),
                            )
                            .children(visible_tasks.iter().map(|(index, task)| {
                                let checkbox = self.row_checkboxes[*index].clone();
                                task_row(task, checkbox, *index, &theme)
                            })),
                    )
                    .child(
                        h_stack()
                            .pt(px(4.))
                            .justify_between()
                            .items_center()
                            .child(
                                div().text_xs().text_color(theme.fg_muted()).child(format!(
                                    "{} of {} selected",
                                    selected_count, total_tasks
                                )),
                            )
                            .child(
                                h_stack()
                                    .gap(px(12.))
                                    .items_center()
                                    .child(div().text_xs().text_color(theme.fg_muted()).child(
                                        format!("Page {} of {}", current_page + 1, total_pages),
                                    ))
                                    .child(
                                        h_stack()
                                            .gap(px(2.))
                                            .child(pagination_button(
                                                "first",
                                                "<<",
                                                current_page > 0,
                                                &theme,
                                                cx.listener(|this, _, _, cx| this.first_page(cx)),
                                            ))
                                            .child(pagination_button(
                                                "prev",
                                                "<",
                                                current_page > 0,
                                                &theme,
                                                cx.listener(|this, _, _, cx| this.prev_page(cx)),
                                            ))
                                            .child(pagination_button(
                                                "next",
                                                ">",
                                                current_page < total_pages - 1,
                                                &theme,
                                                cx.listener(|this, _, _, cx| this.next_page(cx)),
                                            ))
                                            .child(pagination_button(
                                                "last",
                                                ">>",
                                                current_page < total_pages - 1,
                                                &theme,
                                                cx.listener(|this, _, _, cx| this.last_page(cx)),
                                            )),
                                    ),
                            ),
                    ),
            )
    }
}

fn filter_button(
    label: &'static str,
    theme: &std::sync::Arc<gpuikit::theme::Theme>,
) -> impl IntoElement {
    div()
        .id(label)
        .px(px(8.))
        .py(px(4.))
        .bg(gpui::transparent_black())
        .border_1()
        .border_color(theme.border())
        .rounded(px(4.))
        .text_xs()
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface()))
        .child(format!("+ {}", label))
}

fn task_row(
    task: &Task,
    checkbox: Entity<RowCheckbox>,
    index: usize,
    theme: &std::sync::Arc<gpuikit::theme::Theme>,
) -> impl IntoElement {
    let label_variant = match task.label {
        TaskLabel::Bug => BadgeVariant::Destructive,
        TaskLabel::Feature => BadgeVariant::Default,
        TaskLabel::Documentation => BadgeVariant::Secondary,
    };

    let priority_color = match task.priority {
        TaskPriority::High => theme.danger(),
        TaskPriority::Medium => hsla(0.12, 0.8, 0.5, 1.0),
        TaskPriority::Low => theme.fg_muted(),
    };

    h_stack()
        .id(ElementId::NamedInteger("task-row".into(), index as u64))
        .w_full()
        .px(px(8.))
        .py(px(6.))
        .border_b_1()
        .border_color(theme.border())
        .items_center()
        .hover(|s| s.bg(theme.surface()))
        .child(div().w(px(28.)).flex_none().child(checkbox))
        .child(
            div()
                .w(px(80.))
                .flex_none()
                .text_xs()
                .text_color(theme.fg_muted())
                .child(task.id.clone()),
        )
        .child(
            h_stack()
                .flex_1()
                .gap(px(6.))
                .items_center()
                .overflow_hidden()
                .child(badge(task.label.label()).variant(label_variant))
                .child(
                    div()
                        .text_xs()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(task.title.clone()),
                ),
        )
        .child(
            h_stack()
                .w(px(90.))
                .flex_none()
                .gap(px(4.))
                .items_center()
                .text_xs()
                .text_color(theme.fg_muted())
                .child(task.status.icon())
                .child(task.status.label()),
        )
        .child(
            div()
                .w(px(70.))
                .flex_none()
                .text_xs()
                .text_color(priority_color)
                .child(task.priority.label()),
        )
        .child(
            div().w(px(28.)).flex_none().flex().justify_center().child(
                div()
                    .id(ElementId::NamedInteger("task-menu".into(), index as u64))
                    .px(px(4.))
                    .py(px(2.))
                    .rounded(px(2.))
                    .text_xs()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface_secondary()))
                    .child("..."),
            ),
        )
}

fn pagination_button(
    id: &'static str,
    label: &'static str,
    enabled: bool,
    theme: &std::sync::Arc<gpuikit::theme::Theme>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(24.))
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_color(theme.border())
        .rounded(px(3.))
        .text_xs()
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(|s| s.bg(theme.surface()))
                .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                .on_click(on_click)
        })
        .when(!enabled, |this| this.cursor_not_allowed().opacity(0.4))
        .child(label)
}

fn main() {
    Application::new().run(|cx: &mut App| {
        gpuikit::init(cx);
        let bounds = Bounds::centered(None, size(px(900.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("Tasks")),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| TasksApp::new(window, cx)),
        )
        .expect("Failed to open window");

        cx.activate(true);
    });
}
