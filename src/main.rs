use chrono::Local;
use glib::{MainContext, clone, timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Label, ListBox, ListBoxRow,
    Orientation,
};
use std::cell::RefCell;
use std::rc::Rc;

use chrono::{Duration, NaiveDate};
use dirs::data_local_dir;
use gtk4::DrawingArea;
use gtk4::cairo::Context;
use gtk4::gdk::RGBA;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use std::fs::OpenOptions;
use std::io::Write;

mod audio;

const WORK_SECONDS: i32 = 25 * 60;
const SHORT_BREAK_SECONDS: i32 = 5 * 60;
const LONG_BREAK_SECONDS: i32 = 15 * 60;

const APP_NAME: &str = "disipline-tracker";

fn state_file() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(|| ".".into())
        .join(APP_NAME)
        .join("pomodoro_state.json")
}

fn history_file() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(|| ".".into())
        .join(APP_NAME)
        .join("history.txt")
}

fn ensure_app_dir() {
    if let Some(dir) = state_file().parent() {
        let _ = fs::create_dir_all(dir);
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Mode {
    Work,
    Short,
    Long,
}

#[derive(Serialize, Deserialize)]
struct PomodoroState {
    is_running: bool,
    mode: Mode,
    remaining: i32,
    work_sessions: i32,
    contributions: HashMap<NaiveDate, u32>,
}

impl PomodoroState {
    fn new() -> Self {
        Self {
            is_running: false,
            mode: Mode::Work,
            remaining: WORK_SECONDS,
            work_sessions: 0,
            contributions: HashMap::new(),
        }
    }

    fn start(&mut self) {
        self.is_running = true;
    }
    fn pause(&mut self) {
        self.is_running = false;
    }
    fn reset(&mut self, mode: Mode) {
        self.is_running = false;
        self.mode = mode.clone();
        self.remaining = match mode {
            Mode::Work => WORK_SECONDS,
            Mode::Short => SHORT_BREAK_SECONDS,
            Mode::Long => LONG_BREAK_SECONDS,
        };
    }
}

fn format_time(sec: i32) -> String {
    let m = sec / 60;
    let s = sec % 60;
    format!("{:02}:{:02}", m, s)
}

fn main() {
    // Application id should be reverse domain style
    let app = Application::builder()
        .application_id("org.example.disipline_tracker")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Shared state
    ensure_app_dir();
    let state_file = state_file();
    let mut state = Rc::new(RefCell::new(PomodoroState::new()));
    if state_file.exists() {
        state = Rc::new(RefCell::new(PomodoroState::load_from_file(state_file)));
    }

    // Main window
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(900)
        .default_height(520)
        .title(APP_NAME)
        .build();

    // headerbar
    let header = HeaderBar::builder()
        .name(APP_NAME)
        .show_title_buttons(true)
        .build();
    window.set_titlebar(Some(&header));

    // main horizontal box
    let main_box = GtkBox::new(Orientation::Horizontal, 0);

    // Sidebar (left)
    let sidebar = GtkBox::new(Orientation::Vertical, 6);
    sidebar.set_widget_name("sidebar");
    sidebar.set_margin_start(8);
    sidebar.set_margin_end(8);
    sidebar.set_margin_top(8);
    sidebar.set_margin_bottom(8);
    sidebar.set_width_request(220);

    let proj_label = Label::new(Some("Your Projects"));
    proj_label.set_xalign(0.0);
    sidebar.append(&proj_label);

    let repo_list = ListBox::new();
    for name in ["study-tasks", "thesis", "side-project", "courses", "misc"] {
        let row = ListBoxRow::new();
        let h = GtkBox::new(Orientation::Horizontal, 6);
        let title = Label::new(Some(name));
        title.set_xalign(0.0);
        h.append(&title);
        row.set_child(Some(&h));
        repo_list.append(&row);
    }

    // when row activated: simple info dialog
    let window_clone = window.clone();
    repo_list.connect_row_activated(move |_, row| {
        if let Some(child) = row.child() {
            if let Some(label) = child.first_child() {
                if let Some(lbl) = label.downcast_ref::<Label>() {
                    let dlg = gtk4::MessageDialog::new(
                        Some(&window_clone),
                        gtk4::DialogFlags::MODAL,
                        gtk4::MessageType::Info,
                        gtk4::ButtonsType::Ok,
                        &format!("Selected {}", lbl.text()),
                    );
                    dlg.set_modal(false);
                    dlg.set_secondary_text(Some("This would scope your Pomodoro to this project."));
                    dlg.connect_response(|d, _| d.close());
                    dlg.show();
                }
            }
        }
    });

    sidebar.append(&repo_list);

    let hist_label = Label::new(Some("History"));
    hist_label.set_xalign(0.0);
    sidebar.append(&hist_label);

    let history_list = ListBox::new();
    sidebar.append(&history_list);

    let recent_history = load_recent_history(50);
    for entry in recent_history {
        let row = ListBoxRow::new();
        let label = gtk4::Label::new(Some(&entry));
        label.set_xalign(0.0);
        row.set_child(Some(&label));
        history_list.append(&row);
    }

    // Center area
    let center_area = GtkBox::new(Orientation::Vertical, 10);
    center_area.set_hexpand(true);
    center_area.set_vexpand(true);
    center_area.set_margin_top(20);

    let mode_label = Label::new(Some("Work"));
    mode_label.set_widget_name("mode_label");
    mode_label.set_xalign(0.5);
    center_area.append(&mode_label);

    let timer_label = Label::new(Some(&format_time(WORK_SECONDS)));
    timer_label.set_widget_name("timer_label");
    timer_label.set_xalign(0.5);
    timer_label.set_margin_top(6);
    timer_label.set_margin_bottom(6);
    timer_label.set_wrap(false);
    timer_label.set_halign(gtk4::Align::Center);
    // increase font via CSS? Simpler to set markup
    timer_label.set_markup(&format!(
        "<span size='40000' weight='bold'>{}</span>",
        format_time(WORK_SECONDS)
    ));
    center_area.append(&timer_label);

    // === Heatmap ===
    let heatmap = DrawingArea::new();
    heatmap.set_content_width(53 * 14 + 52 * 3);
    heatmap.set_content_height(7 * 14 + 6 * 3);
    heatmap.set_vexpand(true);
    heatmap.set_hexpand(true);
    center_area.append(&heatmap);

    let state_for_draw = state.clone();
    heatmap.set_draw_func(move |_, cr: &Context, _, _| {
        let cell = 14.0;
        let gap = 3.0;
        let today = Local::today().naive_local();
        let data = &state_for_draw.borrow().contributions;

        for w in 0..53 {
            for d in 0..7 {
                let offset = (w * 7 + d) as i64;
                let day = today - Duration::days(offset);
                let count = data.get(&day).copied().unwrap_or(0);

                let color = match count {
                    0 => RGBA::parse("#161b22").unwrap(),
                    1..=2 => RGBA::parse("#0e4429").unwrap(),
                    3..=4 => RGBA::parse("#006d32").unwrap(),
                    5..=6 => RGBA::parse("#26a641").unwrap(),
                    _ => RGBA::parse("#39d353").unwrap(),
                };

                cr.set_source_rgba(
                    color.red().into(),
                    color.green().into(),
                    color.blue().into(),
                    color.alpha().into(),
                );
                cr.rectangle(w as f64 * (cell + gap), d as f64 * (cell + gap), cell, cell);
                cr.fill().unwrap();
            }
        }
    });

    audio::play_complete_sound();
    // Controls
    let controls = GtkBox::new(Orientation::Horizontal, 6);
    let start_btn = Button::with_label("Start");
    let pause_btn = Button::with_label("Pause");
    let reset_btn = Button::with_label("Reset");
    let skip_btn = Button::with_label("Skip");
    controls.append(&start_btn);
    controls.append(&pause_btn);
    controls.append(&reset_btn);
    controls.append(&skip_btn);
    center_area.append(&controls);

    // Presets
    let presets = GtkBox::new(Orientation::Horizontal, 6);
    let w25 = Button::with_label("Work 25");
    let sb = Button::with_label("Short Break");
    let lb = Button::with_label("Long Break");
    presets.append(&w25);
    presets.append(&sb);
    presets.append(&lb);
    center_area.append(&presets);

    // append to main box
    main_box.append(&sidebar);
    main_box.append(&center_area);

    window.set_child(Some(&main_box));

    // Clone handles for callbacks
    let state_start = state.clone();
    let timer_label_start = timer_label.clone();
    let mode_label_start = mode_label.clone();
    let window_start = window.clone();
    let history_list_start = history_list.clone();

    start_btn.connect_clicked(clone!(@strong state_start => move |_| {
        state_start.borrow_mut().start();
        window_start.set_title(Some("disipline-tracker — running"));
    }));

    let state_pause = state.clone();
    let window_pause = window.clone();
    pause_btn.connect_clicked(move |_| {
        state_pause.borrow_mut().pause();
        window_pause.set_title(Some(APP_NAME));
    });

    let state_reset = state.clone();
    let timer_label_reset = timer_label.clone();
    let mode_label_reset = mode_label.clone();
    reset_btn.connect_clicked(move |_| {
        state_reset.borrow_mut().reset(Mode::Work);
        timer_label_reset.set_markup(&format!(
            "<span size='40000' weight='bold'>{}</span>",
            format_time(WORK_SECONDS)
        ));
        mode_label_reset.set_text("Work");
    });

    let state_skip = state.clone();
    let history_list_skip = history_list.clone();
    let window_skip = window.clone();
    let heatmap_skip = heatmap.clone();
    skip_btn.connect_clicked(clone!(@strong state_skip => move |_| {
        complete_session(&state_skip, &history_list_skip, &heatmap_skip, &window_skip);
    }));

    let state_w25 = state.clone();
    let timer_label_w25 = timer_label.clone();
    let mode_label_w25 = mode_label.clone();
    w25.connect_clicked(move |_| {
        state_w25.borrow_mut().reset(Mode::Work);
        timer_label_w25.set_markup(&format!(
            "<span size='40000' weight='bold'>{}</span>",
            format_time(WORK_SECONDS)
        ));
        mode_label_w25.set_text("Work");
    });

    let state_sb = state.clone();
    let timer_label_sb = timer_label.clone();
    let mode_label_sb = mode_label.clone();
    sb.connect_clicked(move |_| {
        state_sb.borrow_mut().reset(Mode::Short);
        timer_label_sb.set_markup(&format!(
            "<span size='40000' weight='bold'>{}</span>",
            format_time(SHORT_BREAK_SECONDS)
        ));
        mode_label_sb.set_text("Short Break");
    });

    let state_lb = state.clone();
    let timer_label_lb = timer_label.clone();
    let mode_label_lb = mode_label.clone();
    lb.connect_clicked(move |_| {
        state_lb.borrow_mut().reset(Mode::Long);
        timer_label_lb.set_markup(&format!(
            "<span size='40000' weight='bold'>{}</span>",
            format_time(LONG_BREAK_SECONDS)
        ));
        mode_label_lb.set_text("Long Break");
    });

    // Tick every second
    let state_tick = state.clone();
    let timer_label_tick = timer_label.clone();
    let mode_label_tick = mode_label.clone();
    let history_list_tick = history_list.clone();
    let window_tick = window.clone();
    let heatmap_tick = heatmap.clone();

    // Use local main context so GTK calls are in main thread
    timeout_add_seconds_local(1, move || {
        let mut s = state_tick.borrow_mut();
        if s.is_running {
            s.remaining -= 1;
            if s.remaining <= 0 {
                // complete session
                s.remaining = 0;
                s.is_running = false;
                // record and switch
                complete_session_internal(&mut s, &history_list_tick, &heatmap_tick, &window_tick);
            }
        }
        // update UI labels
        timer_label_tick.set_markup(&format!(
            "<span size='40000' weight='bold'>{}</span>",
            format_time(s.remaining)
        ));
        mode_label_tick.set_text(match s.mode {
            Mode::Work => "Work",
            Mode::Short => "Short Break",
            Mode::Long => "Long Break",
        });

        glib::Continue(true)
    });

    window.show();
}

fn complete_session(
    state_rc: &Rc<RefCell<PomodoroState>>,
    history_list: &ListBox,
    heatmap: &DrawingArea,
    window: &ApplicationWindow,
) {
    // convenience wrapper to borrow mut and call internal
    let mut s = state_rc.borrow_mut();
    complete_session_internal(&mut s, history_list, heatmap, window);
}

fn complete_session_internal(
    s: &mut PomodoroState,
    history_list: &ListBox,
    heatmap: &DrawingArea,
    window: &ApplicationWindow,
) {
    let mode = s.mode.clone();
    let ts = Local::now();
    let label_text = format!(
        "{} — {}",
        ts.format("%Y-%m-%d %H:%M:%S"),
        match mode {
            Mode::Work => "Work",
            Mode::Short => "Short Break",
            Mode::Long => "Long Break",
        }
    );

    // prepend to history list
    let row = ListBoxRow::new();
    let label = Label::new(Some(&label_text));
    label.set_xalign(0.0);
    row.set_child(Some(&label));
    // ListBox has no prepend method in gtk4-rs so insert at 0
    history_list.insert(&row, 0);
    append_history_file(&label_text);

    // update sessions and decide next mode
    if mode == Mode::Work {
        s.work_sessions += 1;
        let day = Local::today().naive_local();
        *s.contributions.entry(day).or_insert(0) += 1;
        heatmap.queue_draw();
        audio::play_complete_sound();
        if s.work_sessions % 4 == 0 {
            s.reset(Mode::Long);
        } else {
            s.reset(Mode::Short);
        }
    } else {
        s.reset(Mode::Work);
    }

    PomodoroState::save_to_file(s, state_file());
    // simple notification dialog
    let dlg = gtk4::MessageDialog::new(
        Some(window),
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Info,
        gtk4::ButtonsType::Ok,
        "Session complete",
    );
    dlg.set_secondary_text(Some("Session finished — switching modes."));
    dlg.connect_response(|d, _| d.close());
    dlg.show();
}

use std::fs;

impl PomodoroState {
    fn save_to_file(&self, path: PathBuf) {
        if let Ok(json) = serde_json::to_string(self) {
            let _ = fs::write(path, json);
        }
    }

    fn load_from_file(path: PathBuf) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str::<PomodoroState>(&data) {
                return state;
            }
        }
        PomodoroState::new()
    }
}

fn append_history_file(entry: &str) {
    ensure_app_dir();
    let path = history_file();
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{}", entry);
    }
}

fn load_recent_history(limit: usize) -> Vec<String> {
    let path = history_file();
    if let Ok(content) = std::fs::read_to_string(&path) {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        lines.reverse(); // most recent first
        lines.into_iter().take(limit).collect()
    } else {
        Vec::new()
    }
}
