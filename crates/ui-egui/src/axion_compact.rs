//! Axion DocX on a tablet or a phone (the approved Axion layouts): a top bar with the document's
//! name and its few actions, pill tabs, and a compact Home ribbon with large touch targets. On a
//! phone the ribbon is a sheet at the bottom of the screen. The other tabs show their full ribbon
//! groups in a strip that scrolls sideways, so every command stays within reach.

use egui::{Align, Align2, Color32, CornerRadius, Layout, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::{Value, json};

use crate::WordApp;
use crate::icons;
use crate::theme::{Tokens, medium, regular, semibold};

/// Below this width the window is a phone.
pub const PHONE: f32 = 700.0;
/// Below this width (and above a phone) the window is a tablet.
pub const TABLET: f32 = 1100.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Phone,
    Tablet,
    Desktop,
}

pub fn size(ctx: &egui::Context) -> Size {
    let w = ctx.content_rect().width();
    if w < PHONE {
        Size::Phone
    } else if w < TABLET {
        Size::Tablet
    } else {
        Size::Desktop
    }
}

/// The tabs a compact window offers (the rest are a scroll away on a desktop).
const TABLET_TABS: [&str; 8] = ["Home", "Insert", "Draw", "Design", "Layout", "References", "Review", "View"];
const PHONE_TABS: [&str; 6] = ["Home", "Insert", "Layout", "References", "Review", "View"];

fn card_fill(t: &Tokens) -> Color32 {
    if t.dark { t.panel } else { Color32::WHITE }
}

fn lift(t: &Tokens) -> egui::Shadow {
    egui::Shadow { offset: [0, 4], blur: 16, spread: 0, color: t.page_shadow }
}

/// A round white button with an icon (undo, search, share), 44 points.
fn round_button(ui: &mut Ui, t: &Tokens, icon: &str, tip: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tip));
    let fill = if resp.is_pointer_button_down_on() {
        t.pressed
    } else if resp.hovered() {
        t.hover
    } else {
        card_fill(t)
    };
    ui.painter().add(egui::Shadow { offset: [0, 1], blur: 4, spread: 0, color: t.page_shadow }.as_shape(rect, CornerRadius::same(22)));
    ui.painter().rect_filled(rect, 22.0, fill);
    icons::paint(ui.painter(), Rect::from_center_size(rect.center(), vec2(19.0, 19.0)), icon, t.icon, t.accent);
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The blue "Save As" pill and its menu (Word, PDF, Markdown, more formats).
fn save_as_pill(app: &mut WordApp, ui: &mut Ui, t: &Tokens, compact: bool) {
    let label = tl!("Save As");
    let font = semibold(14.5);
    let w = ui.fonts_mut(|f| f.layout_no_wrap(label.to_string(), font.clone(), Color32::WHITE).size().x);
    let size = if compact { vec2(44.0, 44.0) } else { vec2(w + 52.0, 44.0) };
    let (r, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    let fill = if resp.is_pointer_button_down_on() || resp.hovered() { t.accent_text } else { t.accent };
    if compact {
        ui.painter().add(egui::Shadow { offset: [0, 1], blur: 4, spread: 0, color: t.page_shadow }.as_shape(r, CornerRadius::same(22)));
        ui.painter().rect_filled(r, 22.0, if resp.hovered() { t.hover } else { card_fill(t) });
        icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(19.0, 19.0)), "share", t.icon, t.accent);
    } else {
        ui.painter()
            .add(egui::Shadow { offset: [0, 6], blur: 16, spread: 0, color: t.accent.gamma_multiply(0.3) }.as_shape(r, CornerRadius::same(22)));
        ui.painter().rect_filled(r, 22.0, fill);
        ui.painter().text(pos2(r.min.x + 18.0, r.center().y), Align2::LEFT_CENTER, label, font, t.on_accent);
        let c = pos2(r.max.x - 20.0, r.center().y);
        ui.painter().add(egui::Shape::line(vec![c + vec2(-4.0, -2.0), c + vec2(0.0, 2.0), c + vec2(4.0, -2.0)], Stroke::new(1.6, t.on_accent)));
    }
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    egui::Popup::menu(&resp).gap(8.0).align(egui::RectAlign::BOTTOM_END).show(|ui| {
        ui.set_min_width(270.0);
        for (badge, label, hint, ext) in [
            ("DOCX", "Word document", "Keeps every format", "docx"),
            ("PDF", "PDF", "To send, print or file", "pdf"),
            ("MD", "Markdown", "Clean text for AI and notes", "md"),
        ] {
            if crate::widgets::format_row(ui, badge, tl!(label), tl!(hint)).clicked() {
                if ext == "docx" {
                    let _ = app.save_as(crate::file_dialogs::AfterSave::Nothing);
                } else {
                    app.export_dialog(ext);
                }
                ui.close();
            }
        }
        ui.separator();
        if ui.add(egui::Button::new(egui::RichText::new(tl!("More formats…")).font(medium(13.5))).frame(false)).clicked() {
            let _ = app.run("ui.backstage", json!({"value": true, "page": "export"}));
            ui.close();
        }
    });
}

fn status_line(app: &WordApp) -> (String, bool) {
    if !app.has_document() {
        return (String::new(), false);
    }
    if app.autosaves() {
        (tl!("AutoSave on").to_string(), true)
    } else if app.session.dirty {
        (tl!("Edited").to_string(), false)
    } else if app.session.path.is_some() {
        (tl!("Saved").to_string(), false)
    } else {
        (tl!("Not saved").to_string(), false)
    }
}

/// Tablet: mark, title and status on the left; undo, search and Save As on the right.
pub fn tablet_top(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("axion_tablet_top")
        .exact_size(72.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.title_bar).inner_margin(egui::Margin { left: 16, right: 16, top: 10, bottom: 6 }))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let (mark, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
                crate::chrome::axion_mark(ui.painter(), mark);
                let title = if app.has_document() { app.title_stem() } else { "Axion DocX".to_string() };
                let (status, on) = status_line(app);
                let room = (ui.available_width() - 260.0).max(80.0);
                ui.allocate_ui_with_layout(vec2(room, 44.0), Layout::top_down(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.add(egui::Label::new(egui::RichText::new(title).font(semibold(16.0)).color(t.text)).truncate().selectable(false));
                    if !status.is_empty() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            if on {
                                let (r, _) = ui.allocate_exact_size(vec2(8.0, 14.0), Sense::hover());
                                ui.painter().circle_filled(r.center(), 3.5, t.green);
                            }
                            ui.add(egui::Label::new(egui::RichText::new(status).font(regular(12.5)).color(t.text_dim)).selectable(false));
                        });
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_enabled_ui(app.has_document(), |ui| save_as_pill(app, ui, &t, false));
                    if round_button(ui, &t, "search", tl!("Search")).clicked() {
                        let _ = app.run("tools.search", json!({}));
                    }
                    if ui.add_enabled_ui(app.has_document(), |ui| round_button(ui, &t, "undo", tl!("Undo"))).inner.clicked() {
                        let _ = app.run("edit.undo", json!({}));
                    }
                });
            });
        });
}

/// Phone: Arquivo (a blue round button), the title with the page, undo and Save As.
pub fn phone_top(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("axion_phone_top")
        .exact_size(64.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.canvas).inner_margin(egui::Margin { left: 12, right: 12, top: 8, bottom: 4 }))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (r, resp) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("File")));
                ui.painter().add(
                    egui::Shadow { offset: [0, 6], blur: 14, spread: 0, color: t.accent.gamma_multiply(0.3) }.as_shape(r, CornerRadius::same(22)),
                );
                ui.painter().rect_filled(r, 22.0, if resp.hovered() { t.accent_text } else { t.accent });
                paint_folder(ui.painter(), r.center(), t.on_accent);
                if resp.on_hover_text(tl!("File")).clicked() {
                    let _ = app.run("ui.backstage", json!({"value": true}));
                }
                let room = (ui.available_width() - 2.0 * 52.0).max(60.0);
                let title = if app.has_document() { app.title_stem() } else { "Axion DocX".to_string() };
                let (status, _) = status_line(app);
                let pages = if app.has_document() { app.session.layout().pages.len() } else { 0 };
                let page = app.canvas.viewed_page.map_or(1, |p| p + 1).min(pages.max(1));
                let detail = if pages > 0 {
                    format!(
                        "{status} · {}",
                        crate::i18n::fmt(tl!("Page {page} of {pages}"), &[("page", &page.to_string()), ("pages", &pages.to_string())])
                    )
                } else {
                    status
                };
                ui.allocate_ui_with_layout(vec2(room, 44.0), Layout::top_down(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.add(egui::Label::new(egui::RichText::new(title).font(semibold(16.0)).color(t.text)).truncate().selectable(false));
                    ui.add(egui::Label::new(egui::RichText::new(detail).font(regular(12.5)).color(t.text_dim)).truncate().selectable(false));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_enabled_ui(app.has_document(), |ui| save_as_pill(app, ui, &t, true));
                    if ui.add_enabled_ui(app.has_document(), |ui| round_button(ui, &t, "undo", tl!("Undo"))).inner.clicked() {
                        let _ = app.run("edit.undo", json!({}));
                    }
                });
            });
        });
}

/// The pill tabs: File (blue) first on a tablet, then the tabs; the current one is raised.
fn tab_pills(app: &mut WordApp, ui: &mut Ui, t: &Tokens, phone: bool) {
    let tabs: &[&str] = if phone { &PHONE_TABS } else { &TABLET_TABS };
    egui::ScrollArea::horizontal().id_salt("axion-compact-tabs").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(
        ui,
        |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if !phone {
                    let (r, resp) = pill_tab(ui, t, tl!("File"), false, true);
                    let _ = r;
                    if resp.clicked() {
                        let _ = app.run("ui.backstage", json!({"value": true}));
                    }
                }
                for tab in tabs {
                    let active = app.ui.tab == *tab;
                    let (_, resp) = pill_tab(ui, t, tl!(tab), active, false);
                    if resp.clicked() {
                        let _ = app.run("ui.tab", json!({"tab": tab}));
                    }
                }
            });
        },
    );
}

fn pill_tab(ui: &mut Ui, t: &Tokens, label: &str, active: bool, file: bool) -> (Rect, egui::Response) {
    let font = if active || file { semibold(14.0) } else { regular(14.0) };
    let w = ui.fonts_mut(|f| f.layout_no_wrap(label.to_string(), font.clone(), t.text).size().x);
    let (r, resp) = ui.allocate_exact_size(vec2(w + 30.0, 38.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, label));
    let color = if file {
        ui.painter().rect_filled(r, 19.0, if resp.hovered() { t.accent_text } else { t.accent });
        t.on_accent
    } else if active {
        ui.painter().add(egui::Shadow { offset: [0, 2], blur: 8, spread: 0, color: t.page_shadow }.as_shape(r, CornerRadius::same(19)));
        ui.painter().rect_filled(r, 19.0, if t.dark { t.hover } else { t.checked });
        t.accent_text
    } else {
        if resp.hovered() {
            ui.painter().rect_filled(r, 19.0, t.hover);
        }
        t.text
    };
    ui.painter().text(r.center(), Align2::CENTER_CENTER, label, font, color);
    (r, resp.on_hover_cursor(egui::CursorIcon::PointingHand))
}

/// A touch button of the compact ribbon: `checked` gets the white raised look inside a segment.
fn tbtn(ui: &mut Ui, app: &mut WordApp, icon: &str, tip: &str, id: &str, params: Value, checked: bool, size: f32) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let tip = tl!(tip);
    let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), checked, tip));
    let on = id.is_empty() || crate::widgets::enabled(app, id);
    if checked {
        ui.painter().add(egui::Shadow { offset: [0, 1], blur: 4, spread: 0, color: t.page_shadow }.as_shape(r, CornerRadius::same(10)));
        ui.painter().rect_filled(r, 10.0, card_fill(&t));
    } else if resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(r, 10.0, t.pressed);
    } else if resp.hovered() {
        ui.painter().rect_filled(r, 10.0, t.hover);
    }
    let ink = if !on {
        t.text_disabled
    } else if checked {
        t.accent_text
    } else {
        t.icon
    };
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(19.0, 19.0)), icon, ink, t.accent);
    let resp = resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() && on && !id.is_empty() {
        crate::widgets::run_from_ribbon(app, id, params);
    }
    resp
}

/// A soft grey segment holding a few buttons (bold/italic/underline, the alignments).
fn segment<R>(ui: &mut Ui, t: &Tokens, n: usize, size: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let w = n as f32 * size + (n.saturating_sub(1)) as f32 * 2.0 + 8.0;
    let (r, _) = ui.allocate_exact_size(vec2(w, size + 8.0), Sense::hover());
    ui.painter().rect_filled(r, 14.0, if t.dark { t.hover } else { t.canvas });
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r.shrink(4.0)).layout(Layout::left_to_right(Align::Center)));
    child.spacing_mut().item_spacing.x = 2.0;
    add(&mut child)
}

/// A style chip (Normal, Heading 1): the current one has the blue edge.
fn style_chip(ui: &mut Ui, app: &mut WordApp, t: &Tokens, id: &str, current: &str, height: f32) {
    let name = app.session.doc.styles.gallery().iter().find(|s| s.id == id).map(|s| crate::i18n::style_name(s)).unwrap_or_else(|| id.to_string());
    let active = current == id;
    let font = if id.starts_with("Heading") { semibold(13.5) } else { regular(13.5) };
    let w = ui.fonts_mut(|f| f.layout_no_wrap(name.clone(), font.clone(), t.text).size().x);
    let (r, resp) = ui.allocate_exact_size(vec2(w + 28.0, height), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, &name));
    let fill = if resp.hovered() {
        t.hover
    } else if active {
        t.checked
    } else {
        card_fill(t)
    };
    ui.painter().rect(r, 12.0, fill, Stroke::new(if active { 2.0 } else { 1.0 }, if active { t.accent } else { t.border }), egui::StrokeKind::Inside);
    let ink = if id.starts_with("Heading") { t.accent_text } else { t.text };
    ui.painter().text(r.center(), Align2::CENTER_CENTER, &name, font, ink);
    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        let _ = app.run("para.style", json!({"style": id}));
        app.canvas.want_focus = true;
    }
}

/// The compact Home rows. A tablet has Paste and two rows; a phone, two rows of the essentials.
fn compact_home(app: &mut WordApp, ui: &mut Ui, phone: bool) {
    let t = Tokens::get(ui.ctx());
    let st = app.session.run("format.state", &json!({})).unwrap_or_default();
    let flag = |k: &str| st.get(k).and_then(Value::as_bool).unwrap_or(false);
    let font = st.get("font").and_then(Value::as_str).unwrap_or("").to_string();
    let size = st.get("size").and_then(Value::as_f64).map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { format!("{s}") }).unwrap_or_default();
    let style = st.get("style").and_then(Value::as_str).unwrap_or("Normal").to_string();
    let rp = app.session.doc.para_at(&app.session.sel.focus).map(|p| app.session.doc.styles.resolve_para(&p.props));
    let align = rp.as_ref().map(|r| r.align.visual(r.bidi)).unwrap_or_default();
    let lists = wordcraft_engine::cmd::para::list_buttons(&app.session.doc, rp.as_ref().and_then(|r| r.numbering));
    let fc = app.canvas.last_font_color.clone();
    let b = 40.0;
    let font_row = |ui: &mut Ui, app: &mut WordApp, width: f32| {
        let fams = app.previews.families();
        let shown = app.previews.font_shown(&font);
        let recent: Vec<String> = app.ui.recent_fonts.iter().map(|f| app.previews.font_shown(f)).collect();
        let prev = app.previews.font_preview_fn();
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 40.0;
            if let Some(f) = crate::widgets::font_combo_with_recent_hint(ui, "axion-font", width, &shown, &fams, &recent, Some(&*prev), tl!("Font")) {
                let f = app.previews.font_stored(&f);
                let _ = app.run("format.font", json!({"name": f}));
            }
            let sizes: Vec<String> =
                wordcraft_engine::cmd::format::SIZES.iter().map(|s| if s.fract() == 0.0 { format!("{s:.0}") } else { s.to_string() }).collect();
            if let Some(v) = crate::widgets::combo_hint(ui, "axion-size", 58.0, &size, &sizes, None, tl!("Font Size"))
                && let Ok(x) = v.trim().parse::<f64>()
            {
                let _ = app.run("format.size", json!({"size": x}));
            }
        });
    };
    let align_seg = |ui: &mut Ui, app: &mut WordApp| {
        use wordcraft_doc::Align as A;
        segment(ui, &t, 4, b - 6.0, |ui| {
            tbtn(ui, app, "alignLeft", "Align Left", "para.alignLeft", json!({}), align == A::Left, b - 6.0);
            tbtn(ui, app, "alignCenter", "Center", "para.alignCenter", json!({}), align == A::Center, b - 6.0);
            tbtn(ui, app, "alignRight", "Align Right", "para.alignRight", json!({}), align == A::Right, b - 6.0);
            tbtn(ui, app, "justify", "Justify", "para.justify", json!({}), align == A::Justify, b - 6.0);
        });
    };
    if phone {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let w = (ui.available_width() - 58.0 - 110.0).max(110.0);
            font_row(ui, app, w);
            style_chip(ui, app, &t, &style, &style, 40.0);
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            tbtn(ui, app, "bold", "Bold", "format.bold", json!({}), flag("bold"), 44.0);
            tbtn(ui, app, "italic", "Italic", "format.italic", json!({}), flag("italic"), 44.0);
            tbtn(ui, app, "underline", "Underline", "format.underline", json!({}), flag("underline"), 44.0);
            tbtn(ui, app, "fontcolor", "Font Color", "format.color", json!({"color": fc}), false, 44.0);
            tbtn(ui, app, "justify", "Justify", "para.justify", json!({}), false, 44.0);
            tbtn(ui, app, "bullets", "Bullets", "para.bullets", json!({}), lists.bullets, 44.0);
            let more = tbtn(ui, app, "more", "More", "", json!({}), false, 44.0);
            egui::Popup::menu(&more).gap(8.0).align(egui::RectAlign::TOP_END).show(|ui| {
                ui.set_min_width(220.0);
                for (label, id) in [
                    ("Numbering", "para.numbering"),
                    ("Align Left", "para.alignLeft"),
                    ("Center", "para.alignCenter"),
                    ("Strikethrough", "format.strikethrough"),
                    ("Highlight", "format.highlight"),
                    ("Clear All Formatting", "format.clear"),
                    ("Paste", "edit.paste"),
                    ("Font…", "format.fontDialog"),
                    ("Paragraph…", "para.dialog"),
                ] {
                    if ui.add(egui::Button::new(egui::RichText::new(tl!(label)).font(regular(14.5))).min_size(vec2(200.0, 38.0))).clicked() {
                        let _ = app.run(id, json!({}));
                        ui.close();
                    }
                }
            });
        });
        return;
    }
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        // Paste: the soft blue tile.
        let (r, resp) = ui.allocate_exact_size(vec2(64.0, 2.0 * b + 18.0), Sense::click());
        ui.painter().rect_filled(r, 14.0, if resp.hovered() { t.pressed } else { t.checked });
        icons::paint(ui.painter(), Rect::from_center_size(r.center() - vec2(0.0, 10.0), vec2(26.0, 26.0)), "paste", t.icon, t.accent);
        ui.painter().text(r.center() + vec2(0.0, 18.0), Align2::CENTER_CENTER, tl!("Paste"), semibold(13.0), t.text);
        if resp.on_hover_text(tl!("Paste")).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            crate::widgets::run_from_ribbon(app, "edit.paste", json!({}));
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                font_row(ui, app, 150.0);
                segment(ui, &t, 3, b - 6.0, |ui| {
                    tbtn(ui, app, "bold", "Bold", "format.bold", json!({}), flag("bold"), b - 6.0);
                    tbtn(ui, app, "italic", "Italic", "format.italic", json!({}), flag("italic"), b - 6.0);
                    tbtn(ui, app, "underline", "Underline", "format.underline", json!({}), flag("underline"), b - 6.0);
                });
                tbtn(ui, app, "fontcolor", "Font Color", "format.color", json!({"color": fc}), false, b);
                tbtn(ui, app, "highlight", "Text Highlight Color", "format.highlight", json!({}), false, b);
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                tbtn(ui, app, "bullets", "Bullets", "para.bullets", json!({}), lists.bullets, b);
                tbtn(ui, app, "numbering", "Numbering", "para.numbering", json!({}), lists.numbering, b);
                align_seg(ui, app);
                style_chip(ui, app, &t, "Normal", &style, b);
                style_chip(ui, app, &t, "Heading1", &style, b);
                let (r, resp) = ui.allocate_exact_size(vec2(b, b), Sense::click());
                ui.painter().rect_filled(r, 12.0, if resp.hovered() { t.pressed } else { t.hover });
                icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(16.0, 16.0)), "chevronDown", t.icon, t.accent);
                if resp.on_hover_text(tl!("Styles")).clicked() {
                    let _ = app.run("view.stylesPane", json!({"value": true}));
                }
            });
        });
    });
}

/// The ribbon card: Home is compact; any other tab shows its groups in a sideways strip.
fn ribbon_body(app: &mut WordApp, ui: &mut Ui, phone: bool) {
    if !app.has_document() {
        ui.disable();
    }
    if app.ui.tab == "Home" {
        compact_home(app, ui, phone);
    } else {
        egui::ScrollArea::horizontal().id_salt("axion-compact-ribbon").auto_shrink([false, true]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                crate::ribbon::tab_groups(app, ui);
            });
        });
    }
}

/// Tablet: the tab pills and the ribbon card under the top bar.
pub fn tablet_ribbon(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("axion_tablet_tabs")
        .exact_size(48.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.title_bar).inner_margin(egui::Margin { left: 16, right: 16, top: 2, bottom: 6 }))
        .show(ui, |ui| {
            tab_pills(app, ui, &t, false);
        });
    let home = app.ui.tab == "Home";
    let h = if home { 2.0 * 40.0 + 18.0 + 28.0 } else { crate::widgets::CONTENT_H + crate::widgets::LABEL_H + 28.0 };
    egui::Panel::top("axion_tablet_ribbon")
        .exact_size(h + 10.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.title_bar).inner_margin(egui::Margin { left: 16, right: 16, top: 0, bottom: 10 }))
        .show(ui, |ui| {
            egui::Frame::NONE
                .fill(card_fill(&t))
                .corner_radius(CornerRadius::same(20))
                .stroke(Stroke::new(1.0, t.border))
                .shadow(lift(&t))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    ribbon_body(app, ui, false);
                });
        });
}

/// Phone: the ribbon is a sheet at the bottom: tab pills, then the tab's controls.
pub fn phone_sheet(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let home = app.ui.tab == "Home";
    let h = if home { 38.0 + 40.0 + 44.0 + 2.0 * 10.0 + 24.0 } else { 38.0 + crate::widgets::CONTENT_H + crate::widgets::LABEL_H + 40.0 };
    egui::Panel::bottom("axion_phone_sheet")
        .exact_size(h + 20.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.canvas).inner_margin(egui::Margin { left: 10, right: 10, top: 6, bottom: 14 }))
        .show(ui, |ui| {
            egui::Frame::NONE
                .fill(card_fill(&t))
                .corner_radius(CornerRadius::same(24))
                .shadow(egui::Shadow { offset: [0, 10], blur: 30, spread: 0, color: t.page_shadow.gamma_multiply(1.5) })
                .inner_margin(egui::Margin { left: 10, right: 10, top: 10, bottom: 12 })
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    ui.spacing_mut().item_spacing.y = 10.0;
                    tab_pills(app, ui, &t, true);
                    ribbon_body(app, ui, true);
                });
        });
}

/// A folder outline (the phone's File button), drawn so it takes the button's ink.
fn paint_folder(p: &egui::Painter, c: egui::Pos2, ink: Color32) {
    let pts = vec![
        c + vec2(-9.0, -6.0),
        c + vec2(-3.0, -6.0),
        c + vec2(-1.0, -4.0),
        c + vec2(9.0, -4.0),
        c + vec2(9.0, 7.0),
        c + vec2(-9.0, 7.0),
        c + vec2(-9.0, -6.0),
    ];
    p.add(egui::Shape::line(pts, Stroke::new(1.8, ink)));
    p.line_segment([c + vec2(-9.0, -1.0), c + vec2(9.0, -1.0)], Stroke::new(1.8, ink));
}

/// Whether this window is used with touch (a finger has touched it): drags then scroll.
pub fn touch(ctx: &egui::Context) -> bool {
    let id = egui::Id::new("axion-touch");
    if ctx.data(|d| d.get_temp::<bool>(id)).unwrap_or(false) {
        return true;
    }
    let seen = ctx.input(|i| i.any_touches() || i.events.iter().any(|e| matches!(e, egui::Event::Touch { .. })));
    if seen {
        ctx.data_mut(|d| d.insert_temp(id, true));
    }
    seen
}
