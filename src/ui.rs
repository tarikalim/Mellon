//! The window: folders that hold env files on the left, every `KEY=VALUE` they hold on the right.

use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Icon, IconName, Selectable as _, Sizable as _};
use gpui_kit::component::{Theme, ThemeConfig, ThemeMode};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::config::{self, Config};
use crate::store::{self, Entry};
use crate::theme::pal;

const SIDEBAR: f32 = 260.;
const KEY_COLUMN: f32 = 300.;
const REPO_COLUMN: f32 = 200.;
const ROW: f32 = 34.;
const MASK_MAX: usize = 22;
const MONO: &str = "Menlo";

pub fn run() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            apply_theme(&crate::theme::saved_choice(), cx);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(1200.), px(780.)), cx)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Mellon".into()),
                    ..Default::default()
                }),
                app_id: Some("mellon".into()),
                ..Default::default()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| Mellon::new(window, cx))
            })
            .expect("open window");
            cx.activate(true);
        });
}

/// Draws the window and gpui-component's own widgets (inputs, menus, tooltips) with the named
/// theme; an unknown name falls back to the default.
fn apply_theme(name: &str, cx: &mut App) {
    let themes = crate::theme::available();
    let palette = themes
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or(&themes[0])
        .1;
    crate::theme::set(palette);
    match serde_json::from_value::<ThemeConfig>(crate::theme::component_theme(&palette)) {
        Ok(config) if palette.dark => Theme::global_mut(cx).dark_theme = Rc::new(config),
        Ok(config) => Theme::global_mut(cx).light_theme = Rc::new(config),
        Err(e) => eprintln!("component theme: {e}"),
    }
    let mode = if palette.dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    Theme::change(mode, None, cx);
    cx.refresh_windows();
}

pub struct Mellon {
    config: Config,
    entries: Vec<Entry>,
    /// Indexes into `entries` that pass the search, the folder and the toggles.
    visible: Vec<usize>,
    /// Folders with their key counts, most keys first.
    repos: Vec<(String, usize)>,
    repo: Option<String>,
    search: Entity<InputState>,
    search_values: bool,
    reveal: bool,
    show_commented: bool,
    edit_mode: bool,
    /// The entry whose value is being edited, its field and the field's subscription.
    editing: Option<(usize, Entity<InputState>, Subscription)>,
    adding: bool,
    new_key: Entity<InputState>,
    new_value: Entity<InputState>,
    /// The entry copied last, shown with a check for a moment.
    copied: Option<usize>,
    status: String,
    list: UniformListScrollHandle,
    _subs: [Subscription; 2],
}

impl Mellon {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search keys and folders…"));
        let new_key = cx.new(|cx| InputState::new(window, cx).placeholder("KEY"));
        let new_value = cx.new(|cx| InputState::new(window, cx).placeholder("value"));
        let subs = [
            cx.subscribe_in(&search, window, |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.refilter(cx);
                }
            }),
            cx.subscribe_in(
                &new_value,
                window,
                |this, _, ev: &InputEvent, window, cx| {
                    if let InputEvent::PressEnter { .. } = ev {
                        this.add_to_scratch(window, cx);
                    }
                },
            ),
        ];
        search.update(cx, |s, cx| s.focus(window, cx));
        let config = Config::load();
        let entries = store::scan(&config);
        let mut this = Self {
            config,
            entries,
            visible: Vec::new(),
            repos: Vec::new(),
            repo: None,
            search,
            search_values: false,
            reveal: false,
            show_commented: false,
            edit_mode: false,
            editing: None,
            adding: false,
            new_key,
            new_value,
            copied: None,
            status: String::new(),
            list: UniformListScrollHandle::new(),
            _subs: subs,
        };
        this.recount();
        this.refilter(cx);
        this
    }

    fn recount(&mut self) {
        let mut repos: Vec<(String, usize)> = Vec::new();
        for entry in &self.entries {
            match repos.last_mut() {
                Some((repo, count)) if *repo == entry.repo => *count += 1,
                _ => repos.push((entry.repo.clone(), 1)),
            }
        }
        repos.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        if let Some(r) = &self.repo
            && !repos.iter().any(|(name, _)| name == r)
        {
            self.repo = None;
        }
        self.repos = repos;
    }

    fn refilter(&mut self, cx: &mut Context<Self>) {
        let query = self.search.read(cx).value().to_lowercase();
        let terms: Vec<String> = query.split_whitespace().map(str::to_owned).collect();
        self.visible = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| self.show_commented || !e.commented)
            .filter(|(_, e)| self.repo.as_ref().is_none_or(|r| &e.repo == r))
            .filter(|(_, e)| e.matches(&terms, self.search_values))
            .map(|(i, _)| i)
            .collect();
        cx.notify();
    }

    fn rescan(&mut self, cx: &mut Context<Self>) {
        self.config = Config::load();
        self.entries = store::scan(&self.config);
        self.editing = None;
        self.copied = None;
        self.recount();
        self.status = format!("{} keys", self.entries.len());
        self.refilter(cx);
    }

    fn copy(&mut self, index: usize, cx: &mut Context<Self>) {
        let entry = &self.entries[index];
        cx.write_to_clipboard(ClipboardItem::new_string(entry.value.clone()));
        self.status = format!("Copied {}", entry.key);
        self.copied = Some(index);
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.copied == Some(index) {
                    this.copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }

    fn start_edit(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.as_ref().is_some_and(|(i, ..)| *i == index) {
            return;
        }
        self.save_edit(cx);
        let value = self.entries[index].value.clone();
        let field = cx.new(|cx| InputState::new(window, cx).default_value(value));
        let sub = cx.subscribe_in(&field, window, |this, _, ev: &InputEvent, _, cx| {
            if let InputEvent::PressEnter { .. } | InputEvent::Blur = ev {
                this.save_edit(cx);
            }
        });
        field.update(cx, |s, cx| s.focus(window, cx));
        self.editing = Some((index, field, sub));
        cx.notify();
    }

    /// Writes the edited value back; only that one line of the file is rewritten.
    fn save_edit(&mut self, cx: &mut Context<Self>) {
        let Some((index, field, _)) = self.editing.take() else {
            return;
        };
        let value = field.read(cx).value().to_string();
        if value != self.entries[index].value {
            let entry = &mut self.entries[index];
            entry.value = value;
            entry.refresh_value_index();
            self.status = match store::write_back(entry) {
                Ok(()) => format!("Saved {}", entry.key),
                Err(err) => format!("{}: {err}", entry.key),
            };
        }
        cx.notify();
    }

    fn add_to_scratch(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let key = self.new_key.read(cx).value().trim().to_owned();
        let value = self.new_value.read(cx).value().trim().to_owned();
        if key.is_empty() {
            return;
        }
        match store::append_scratch(&key, &value) {
            Ok(entry) => {
                self.status = format!("Added {}", entry.key);
                self.entries.push(entry);
                self.entries
                    .sort_by(|a, b| (&a.repo, &a.key).cmp(&(&b.repo, &b.key)));
                // Indexes moved with the sort.
                self.editing = None;
                self.copied = None;
                self.new_key.update(cx, |s, cx| s.set_value("", window, cx));
                self.new_value
                    .update(cx, |s, cx| s.set_value("", window, cx));
                self.recount();
                self.refilter(cx);
            }
            Err(err) => {
                self.status = format!("Could not write the scratch file: {err}");
                cx.notify();
            }
        }
    }

    /// In the Mac's default text editor: an app opened from Finder has no `$EDITOR`.
    fn open_path(&mut self, path: &std::path::Path, cx: &mut Context<Self>) {
        let opened = std::process::Command::new("open")
            .arg("-t")
            .arg(path)
            .status();
        if !opened.is_ok_and(|s| s.success()) {
            self.status = format!("Could not open {}", path.display());
            cx.notify();
        }
    }

    fn open_config(&mut self, cx: &mut Context<Self>) {
        match config::write_template_if_missing() {
            Ok(path) => self.open_path(&path, cx),
            Err(err) => {
                self.status = format!("Could not create the config file: {err}");
                cx.notify();
            }
        }
    }
}

impl Render for Mellon {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(pal().bg))
            .text_color(rgb(pal().text))
            .text_size(px(13.))
            .child(self.header(cx))
            .when(self.adding, |el| el.child(self.add_bar(cx)))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .gap(px(10.))
                    .px(px(20.))
                    .child(self.sidebar(cx))
                    .child(self.results(cx)),
            )
            .child(self.footer())
    }
}

impl Mellon {
    /// Search on the left; toggles for what is listed, then the window's actions.
    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(20.))
            .pt(px(18.))
            .pb(px(12.))
            .child(
                div().flex_1().min_w_0().child(
                    Input::new(&self.search)
                        .prefix(Icon::new(IconName::Search).text_color(rgb(pal().muted)))
                        .cleanable(true),
                ),
            )
            .child(div().w(px(8.)))
            .child(
                self.toggle(
                    "reveal",
                    "Reveal",
                    "Show values unmasked",
                    self.reveal,
                    cx,
                    |t| t.reveal = !t.reveal,
                )
                .icon(if self.reveal {
                    IconName::Eye
                } else {
                    IconName::EyeOff
                }),
            )
            .child(self.toggle(
                "values",
                "Values",
                "Search inside values too",
                self.search_values,
                cx,
                |t| t.search_values = !t.search_values,
            ))
            .child(self.toggle(
                "commented",
                "Commented",
                "Also list keys commented out in the files (# KEY=VALUE)",
                self.show_commented,
                cx,
                |t| t.show_commented = !t.show_commented,
            ))
            .child(self.toggle(
                "edit",
                "Edit",
                "Click a row to edit its value in place",
                self.edit_mode,
                cx,
                |t| {
                    t.edit_mode = !t.edit_mode;
                    t.editing = None;
                },
            ))
            .child(div().w(px(8.)))
            .child(
                Button::new("add")
                    .icon(IconName::Plus)
                    .ghost()
                    .selected(self.adding)
                    .tooltip("Add a key that belongs to no folder")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.adding = !this.adding;
                        if this.adding {
                            this.new_key.update(cx, |s, cx| s.focus(window, cx));
                        }
                        cx.notify();
                    })),
            )
            .child(
                Button::new("rescan")
                    .icon(IconName::RefreshCw)
                    .ghost()
                    .tooltip("Rescan the folders and reload the config")
                    .on_click(cx.listener(|this, _, _, cx| this.rescan(cx))),
            )
            .child(theme_menu())
            .child(
                Button::new("config")
                    .icon(IconName::Settings)
                    .ghost()
                    .tooltip(format!("Open {}", config::config_file().display()))
                    .on_click(cx.listener(|this, _, _, cx| this.open_config(cx))),
            )
    }

    fn toggle(
        &self,
        id: &'static str,
        label: &'static str,
        tip: &'static str,
        on: bool,
        cx: &mut Context<Self>,
        flip: fn(&mut Self),
    ) -> Button {
        Button::new(id)
            .label(label)
            .ghost()
            .selected(on)
            .tooltip(tip)
            .on_click(cx.listener(move |this, _, _, cx| {
                flip(this);
                this.refilter(cx);
            }))
    }

    fn add_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(20.))
            .pb(px(12.))
            .child(div().w(px(KEY_COLUMN)).child(Input::new(&self.new_key)))
            .child(div().flex_1().child(Input::new(&self.new_value)))
            .child(
                Button::new("add-key")
                    .label("Add")
                    .primary()
                    .on_click(cx.listener(|this, _, window, cx| this.add_to_scratch(window, cx))),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(rgb(pal().muted))
                    .child(config::display_path(&config::scratch_file())),
            )
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut list = div()
            .id("folders")
            .flex_1()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(1.));
        for (i, (repo, count)) in self.repos.iter().enumerate() {
            let selected = self.repo.as_ref() == Some(repo);
            let repo = repo.clone();
            list = list.child(
                folder_row(("repo", i), repo.clone(), *count, selected).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.repo = Some(repo.clone());
                        this.refilter(cx);
                    },
                )),
            );
        }
        div()
            .w(px(SIDEBAR))
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(1.))
            .p(px(6.))
            .rounded(px(10.))
            .bg(rgb(pal().surface))
            .child(
                folder_row(
                    "all",
                    "All folders".into(),
                    self.entries.len(),
                    self.repo.is_none(),
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.repo = None;
                    this.refilter(cx);
                })),
            )
            .child(div().my(px(4.)).h(px(1.)).bg(rgb(pal().border)))
            .child(list)
    }

    fn results(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let all = self.repo.is_none();
        let heading = |label: &'static str| {
            div()
                .text_size(px(11.))
                .text_color(rgb(pal().muted))
                .child(label)
        };
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .rounded(px(10.))
            .bg(rgb(pal().surface))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .h(px(30.))
                    .px(px(14.))
                    .border_b_1()
                    .border_color(rgb(pal().border))
                    .child(div().w(px(KEY_COLUMN)).flex_none().child(heading("KEY")))
                    .when(all, |el| {
                        el.child(
                            div()
                                .w(px(REPO_COLUMN))
                                .flex_none()
                                .child(heading("FOLDER")),
                        )
                    })
                    .child(div().flex_1().child(heading("VALUE"))),
            )
            .child(if self.visible.is_empty() {
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(pal().muted))
                    .child("No keys match")
                    .into_any_element()
            } else {
                div()
                    .flex_1()
                    .min_h_0()
                    .child(
                        uniform_list(
                            "rows",
                            self.visible.len(),
                            cx.processor(|this, range, _, cx| this.rows(range, cx)),
                        )
                        .size_full()
                        .track_scroll(&self.list),
                    )
                    .into_any_element()
            })
    }

    /// Click a row to copy its value, or in edit mode to edit it.
    fn rows(&mut self, range: Range<usize>, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let all = self.repo.is_none();
        range
            .map(|i| {
                let index = self.visible[i];
                let entry = &self.entries[index];
                let field = self
                    .editing
                    .as_ref()
                    .filter(|(e, ..)| *e == index)
                    .map(|(_, f, _)| f.clone());
                let key = if entry.commented {
                    div()
                        .italic()
                        .text_color(rgb(pal().muted))
                        .child(format!("# {}", entry.key))
                } else {
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(entry.key.clone())
                };
                let value = match field {
                    Some(field) => Input::new(&field).small().into_any_element(),
                    None => div()
                        .truncate()
                        .font_family(MONO)
                        .text_size(px(12.))
                        .text_color(rgb(if self.reveal { pal().text } else { pal().muted }))
                        .child(if self.reveal {
                            entry.value.clone()
                        } else {
                            mask(&entry.value)
                        })
                        .into_any_element(),
                };
                div()
                    .id(("row", index))
                    .h(px(ROW))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .px(px(14.))
                    .border_b_1()
                    .border_color(rgb(pal().border))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(pal().hover)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if this.edit_mode {
                            this.start_edit(index, window, cx);
                        } else {
                            this.copy(index, cx);
                        }
                    }))
                    .child(
                        div()
                            .w(px(KEY_COLUMN))
                            .flex_none()
                            .min_w_0()
                            .truncate()
                            .child(key),
                    )
                    .when(all, |el| {
                        el.child(
                            div()
                                .w(px(REPO_COLUMN))
                                .flex_none()
                                .truncate()
                                .text_size(px(12.))
                                .text_color(rgb(pal().muted))
                                .child(entry.repo.clone()),
                        )
                    })
                    .child(div().flex_1().min_w_0().child(value))
                    .child(
                        Button::new(("copy", index))
                            .icon(if self.copied == Some(index) {
                                IconName::Check
                            } else {
                                IconName::Copy
                            })
                            .ghost()
                            .xsmall()
                            .tooltip("Copy the value")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.copy(index, cx);
                            })),
                    )
                    .child(
                        Button::new(("open", index))
                            .icon(IconName::ExternalLink)
                            .ghost()
                            .xsmall()
                            .tooltip("Open the file in the default text editor")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                let file = this.entries[index].file.clone();
                                this.open_path(&file, cx);
                            })),
                    )
                    .into_any_element()
            })
            .collect()
    }

    fn footer(&self) -> impl IntoElement {
        let hint = if self.edit_mode {
            "click a row to edit, Enter to save"
        } else {
            "click a row to copy its value"
        };
        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .h(px(32.))
            .px(px(22.))
            .text_size(px(12.))
            .text_color(rgb(pal().muted))
            .child(div().flex_1().truncate().child(self.status.clone()))
            .child(format!(
                "{} of {} keys · {hint}",
                self.visible.len(),
                self.entries.len()
            ))
    }
}

/// Every theme in Zed's theme files; the choice is kept in the config folder.
fn theme_menu() -> impl IntoElement {
    Button::new("theme")
        .icon(IconName::Palette)
        .ghost()
        .tooltip("Theme")
        .dropdown_menu_with_anchor(Anchor::TopRight, |mut menu, _, _| {
            let current = crate::theme::saved_choice();
            for (name, _) in crate::theme::available() {
                let pick = name.clone();
                menu = menu.item(
                    PopupMenuItem::new(name.clone())
                        .checked(name == current)
                        .on_click(move |_, _, cx| {
                            crate::theme::save_choice(&pick);
                            apply_theme(&pick, cx);
                        }),
                );
            }
            menu
        })
}

fn folder_row(
    id: impl Into<ElementId>,
    label: String,
    count: usize,
    selected: bool,
) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .h(px(28.))
        .px(px(10.))
        .rounded(px(6.))
        .cursor_pointer()
        .when(selected, |el| el.bg(rgb(pal().selected)))
        .when(!selected, |el| el.hover(|s| s.bg(rgb(pal().hover))))
        .child(
            Icon::new(IconName::Folder)
                .small()
                .text_color(rgb(pal().muted)),
        )
        .child(div().flex_1().min_w_0().truncate().child(label))
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(pal().muted))
                .child(count.to_string()),
        )
}

fn mask(value: &str) -> String {
    let hidden = value.chars().count().saturating_sub(2).min(MASK_MAX);
    let head: String = value.chars().take(2).collect();
    format!("{head}{}", "•".repeat(hidden))
}

#[cfg(test)]
mod tests {
    use super::mask;

    #[test]
    fn masking_keeps_a_hint_and_caps_the_length() {
        assert_eq!(mask("sk_live_1234"), "sk••••••••••");
        assert_eq!(mask("ab"), "ab");
        assert_eq!(mask(""), "");
        assert_eq!(mask(&"x".repeat(500)).chars().count(), 24);
    }
}
