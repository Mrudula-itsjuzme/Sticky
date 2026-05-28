use crate::db::TextBlock;
use crate::DB;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChecklistItem {
    pub text: String,
    pub checked: bool,
}

glib::wrapper! {
    pub struct TextBlockWidget(ObjectSubclass<imp::TextBlockWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl TextBlockWidget {
    pub fn new(data: TextBlock) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().init_data(data);
        obj
    }

    pub fn get_content(&self) -> String {
        self.imp().data.borrow().content.clone()
    }
}

mod imp {
    use super::*;
    use std::cell::RefCell;

    pub struct TextBlockWidget {
        pub data: RefCell<TextBlock>,
        pub stack: gtk::Stack,
        pub text_view: gtk::TextView,
        pub picture: gtk::Picture,
        pub checklist_box: gtk::Box,
        pub file_box: gtk::Box,
        pub file_btn: gtk::Button,
        pub timer_box: gtk::Box,
        pub timer_label: gtk::Label,
        pub code_view: gtk::TextView,
    }

    impl Default for TextBlockWidget {
        fn default() -> Self {
            let buffer = gtk::TextBuffer::new(None);
            buffer.set_enable_undo(true);
            let text_view = gtk::TextView::builder()
                .wrap_mode(gtk::WrapMode::Word)
                .accepts_tab(false)
                .buffer(&buffer)
                .build();

            let picture = gtk::Picture::builder().can_shrink(true).build();

            let stack = gtk::Stack::new();

            let checklist_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            checklist_box.add_css_class("checklist-box");

            let file_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            let file_btn = gtk::Button::builder()
                .css_classes(["flat", "file-btn"])
                .build();
            file_box.append(&file_btn);
            file_box.set_valign(gtk::Align::Center);

            let timer_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
            let timer_label = gtk::Label::builder()
                .label("25:00")
                .css_classes(["timer-label"])
                .build();
            let timer_btn = gtk::Button::with_label("Start Focus");
            timer_box.append(&timer_label);
            timer_box.append(&timer_btn);
            timer_box.set_valign(gtk::Align::Center);

            let code_buffer = gtk::TextBuffer::new(None);
            let code_view = gtk::TextView::builder()
                .wrap_mode(gtk::WrapMode::None)
                .monospace(true)
                .buffer(&code_buffer)
                .css_classes(["code-view"])
                .build();

            stack.add_child(&text_view);
            stack.add_child(&picture);
            stack.add_child(&checklist_box);
            stack.add_child(&file_box);
            stack.add_child(&timer_box);
            stack.add_child(&code_view);

            Self {
                data: RefCell::new(TextBlock {
                    id: 0,
                    note_id: 0,
                    x: 0.0,
                    y: 0.0,
                    width: 200.0,
                    height: 100.0,
                    content: String::new(),
                }),
                stack,
                text_view,
                picture,
                checklist_box,
                file_box,
                file_btn,
                timer_box,
                timer_label,
                code_view,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TextBlockWidget {
        const NAME: &'static str = "TextBlockWidget";
        type Type = super::TextBlockWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for TextBlockWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_layout_manager(Some(gtk::BinLayout::new()));
            obj.add_css_class("text-block");

            let overlay = gtk::Overlay::new();

            let frame = gtk::Frame::new(None);
            frame.set_child(Some(&self.stack));
            overlay.set_child(Some(&frame));

            // Setup timer button click
            if let Some(btn) = self.timer_box.last_child().and_downcast::<gtk::Button>() {
                let lbl = self.timer_label.clone();
                let obj_weak = obj.downgrade();
                btn.connect_clicked(move |b| {
                    if b.label().as_deref() == Some("Start Focus") {
                        b.set_label("Stop Focus");
                        let start_time = std::time::Instant::now();
                        let l = lbl.clone();
                        let btn_weak = b.downgrade();
                        let o_weak = obj_weak.clone();
                        glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                            if let (Some(b_ref), Some(o_ref)) =
                                (btn_weak.upgrade(), o_weak.upgrade())
                            {
                                if b_ref.label().as_deref() == Some("Stop Focus") {
                                    let elapsed = start_time.elapsed().as_secs();
                                    let total = 25 * 60;
                                    if elapsed >= total {
                                        l.set_label("00:00");
                                        b_ref.set_label("Start Focus");
                                        o_ref.add_css_class("timer-done");
                                        return glib::ControlFlow::Break;
                                    } else {
                                        let rem = total - elapsed;
                                        l.set_label(&format!("{:02}:{:02}", rem / 60, rem % 60));
                                        return glib::ControlFlow::Continue;
                                    }
                                }
                            }
                            glib::ControlFlow::Break
                        });
                    } else {
                        b.set_label("Start Focus");
                        lbl.set_label("25:00");
                        if let Some(o) = obj_weak.upgrade() {
                            o.remove_css_class("timer-done");
                        }
                    }
                });
            }

            let delete_btn = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .css_classes(["delete-btn", "flat"])
                .halign(gtk::Align::End)
                .valign(gtk::Align::Start)
                .build();

            delete_btn.connect_clicked(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.add_css_class("peel-out");
                    let obj_weak = obj.downgrade();
                    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                        if let Some(obj) = obj_weak.upgrade() {
                            if let Some(canvas) =
                                obj.parent().and_downcast::<crate::canvas::Canvas>()
                            {
                                canvas.remove_block(&obj);
                            }
                        }
                        glib::ControlFlow::Break
                    });
                }
            ));

            overlay.add_overlay(&delete_btn);

            let speak_btn = gtk::Button::builder()
                .icon_name("audio-volume-high-symbolic")
                .css_classes(["delete-btn", "flat"])
                .halign(gtk::Align::End)
                .valign(gtk::Align::End)
                .build();

            speak_btn.connect_clicked(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    let text = obj.get_content();
                    if !text.is_empty() {
                        let clean_text = text.replace("[CHECKLIST]", "").replace("\"", "");
                        std::thread::spawn(move || {
                            let _ = std::process::Command::new("spd-say")
                                .arg(&clean_text)
                                .output();
                        });
                    }
                }
            ));

            overlay.add_overlay(&speak_btn);

            let link_btn = gtk::Button::builder()
                .icon_name("insert-link-symbolic")
                .css_classes(["delete-btn", "flat"])
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            link_btn.connect_clicked(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    if let Some(canvas) = obj.parent().and_downcast::<crate::canvas::Canvas>() {
                        canvas.start_linking(obj.imp().data.borrow().id);
                    }
                }
            ));
            overlay.add_overlay(&link_btn);

            overlay.set_parent(&*obj);

            let drag = gtk::GestureDrag::new();
            obj.add_controller(drag.clone());

            drag.connect_drag_update(glib::clone!(
                #[weak]
                obj,
                move |drag, offset_x, offset_y| {
                    if let Some(parent) = obj.parent().and_downcast::<gtk::Fixed>() {
                        let data = obj.imp().data.borrow();
                        let new_x = data.x + offset_x;
                        let new_y = data.y + offset_y;
                        parent.move_(&obj, new_x, new_y);
                        drag.set_state(gtk::EventSequenceState::Claimed);
                    }
                }
            ));

            drag.connect_drag_end(glib::clone!(
                #[weak]
                obj,
                move |_, offset_x, offset_y| {
                    {
                        let mut data = obj.imp().data.borrow_mut();
                        data.x += offset_x;
                        data.y += offset_y;
                    }
                    obj.imp().save_data();
                }
            ));

            let buffer = self.text_view.buffer();
            buffer.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.imp().save_data();
                }
            ));

            // Keyboard shortcuts and math evaluation
            let key_controller = gtk::EventControllerKey::new();
            let obj_weak = obj.downgrade();
            key_controller.connect_key_pressed(move |_, key, _keycode, state| {
                let Some(obj) = obj_weak.upgrade() else { return glib::Propagation::Proceed; };
                let text_view = &obj.imp().text_view;
                let buffer = text_view.buffer();

                // Markdown wrapping shortcuts
                if state.contains(gdk::ModifierType::CONTROL_MASK) {
                    match key {
                        gdk::Key::b | gdk::Key::B => {
                            wrap_selection(&buffer, "**", "**");
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::i | gdk::Key::I => {
                            wrap_selection(&buffer, "*", "*");
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::u | gdk::Key::U => {
                            wrap_selection(&buffer, "<u>", "</u>");
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::h | gdk::Key::H => {
                            wrap_selection(&buffer, "<mark>", "</mark>");
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::g | gdk::Key::G => {
                            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
                            let mut end_iter = buffer.end_iter();
                            buffer.insert(&mut end_iter, "\n[Generating...]");

                            let buffer_clone = buffer.clone();
                            glib::MainContext::default().spawn_local(async move {
                                let result = crate::TOKIO_RT.spawn(async move {
                                    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
                                    if api_key.is_empty() {
                                        return "Error: OPENAI_API_KEY not set".to_string();
                                    }
                                    let payload = serde_json::json!({
                                        "model": "gpt-4o-mini",
                                        "messages": [
                                            {"role": "system", "content": "You are a helpful AI assistant built into a sticky note. Continue, complete, or fulfill the prompt provided by the user. Keep it concise."},
                                            {"role": "user", "content": text}
                                        ]
                                    });
                                    let client = reqwest::Client::new();
                                    let res = client.post("https://api.openai.com/v1/chat/completions")
                                        .bearer_auth(&api_key)
                                        .json(&payload)
                                        .send()
                                        .await;

                                    if let Ok(r) = res {
                                        if let Ok(json) = r.json::<serde_json::Value>().await {
                                            json["choices"][0]["message"]["content"].as_str().unwrap_or("Error parsing JSON").to_string()
                                        } else { "Error parsing JSON payload".to_string() }
                                    } else { "API request failed".to_string() }
                                }).await.unwrap_or_else(|_| "Task panicked".to_string());

                                let mut full_text = buffer_clone.text(&buffer_clone.start_iter(), &buffer_clone.end_iter(), false).to_string();
                                full_text = full_text.replace("\n[Generating...]", &format!("\n\n{}", result));
                                buffer_clone.set_text(&full_text);
                            });
                            return glib::Propagation::Stop;
                        }
                        _ => {}
                    }
                }

                // Inline math calculator
                if key == gdk::Key::equal {
                    let mut insert_iter = buffer.iter_at_mark(&buffer.get_insert());
                    let mut start_iter = insert_iter;

                    // Find the start of the expression (space or newline)
                    while start_iter.backward_char() {
                        let c = start_iter.char();
                        if c.is_whitespace() || c == '\n' {
                            start_iter.forward_char();
                            break;
                        }
                    }

                    let expr_text = buffer.text(&start_iter, &insert_iter, false).to_string();
                    if !expr_text.trim().is_empty() {
                        if let Ok(result) = evalexpr::eval(&expr_text) {
                            let result_str = result.to_string();
                            buffer.begin_user_action();
                            buffer.delete(&mut start_iter, &mut insert_iter);
                            buffer.insert(&mut start_iter, &result_str);
                            buffer.end_user_action();
                            return glib::Propagation::Stop;
                        }
                    }
                }

                glib::Propagation::Proceed
            });
            self.text_view.add_controller(key_controller);

            // Setup Markdown Tags
            let tag_table = buffer.tag_table();
            let bold_tag = gtk::TextTag::builder().name("bold").weight(700).build();
            let italic_tag = gtk::TextTag::builder()
                .name("italic")
                .style(gtk::pango::Style::Italic)
                .build();
            let hide_tag = gtk::TextTag::builder().name("hide").invisible(true).build();
            tag_table.add(&bold_tag);
            tag_table.add(&italic_tag);
            tag_table.add(&hide_tag);

            // Focus controller for WYSIWYG
            let focus_ctrl = gtk::EventControllerFocus::new();
            focus_ctrl.connect_enter(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    let text_view = &obj.imp().text_view;
                    let buffer = text_view.buffer();
                    buffer.remove_all_tags(&buffer.start_iter(), &buffer.end_iter());
                }
            ));
            focus_ctrl.connect_leave(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.imp().apply_markdown();
                }
            ));
            self.text_view.add_controller(focus_ctrl);
        }

        fn dispose(&self) {
            if let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl TextBlockWidget {
        fn apply_markdown(&self) {
            let buffer = self.text_view.buffer();
            buffer.remove_all_tags(&buffer.start_iter(), &buffer.end_iter());
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string();
            let chars: Vec<char> = text.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
                    let mut j = i + 2;
                    let mut found = false;
                    while j + 1 < chars.len() {
                        if chars[j] == '*' && chars[j + 1] == '*' {
                            found = true;
                            break;
                        }
                        j += 1;
                    }
                    if found {
                        let s_h1 = buffer.iter_at_offset(i as i32);
                        let e_h1 = buffer.iter_at_offset((i + 2) as i32);
                        let s_b = buffer.iter_at_offset((i + 2) as i32);
                        let e_b = buffer.iter_at_offset(j as i32);
                        let s_h2 = buffer.iter_at_offset(j as i32);
                        let e_h2 = buffer.iter_at_offset((j + 2) as i32);

                        buffer.apply_tag_by_name("hide", &s_h1, &e_h1);
                        buffer.apply_tag_by_name("bold", &s_b, &e_b);
                        buffer.apply_tag_by_name("hide", &s_h2, &e_h2);
                        i = j + 2;
                        continue;
                    }
                } else if chars[i] == '*' {
                    let mut j = i + 1;
                    let mut found = false;
                    while j < chars.len() {
                        if chars[j] == '*' {
                            found = true;
                            break;
                        }
                        j += 1;
                    }
                    if found && j > i + 1 {
                        let s_h1 = buffer.iter_at_offset(i as i32);
                        let e_h1 = buffer.iter_at_offset((i + 1) as i32);
                        let s_i = buffer.iter_at_offset((i + 1) as i32);
                        let e_i = buffer.iter_at_offset(j as i32);
                        let s_h2 = buffer.iter_at_offset(j as i32);
                        let e_h2 = buffer.iter_at_offset((j + 1) as i32);

                        buffer.apply_tag_by_name("hide", &s_h1, &e_h1);
                        buffer.apply_tag_by_name("italic", &s_i, &e_i);
                        buffer.apply_tag_by_name("hide", &s_h2, &e_h2);
                        i = j + 1;
                        continue;
                    }
                }
                i += 1;
            }
        }
    }

    fn wrap_selection(buffer: &gtk::TextBuffer, prefix: &str, suffix: &str) {
        buffer.begin_user_action();
        if let Some((mut start, mut end)) = buffer.selection_bounds() {
            let text = buffer.text(&start, &end, false);
            buffer.delete(&mut start, &mut end);
            buffer.insert(&mut start, &format!("{}{}{}", prefix, text, suffix));
        } else {
            let mut iter = buffer.iter_at_mark(&buffer.get_insert());
            buffer.insert(&mut iter, &format!("{}{}", prefix, suffix));
            let new_offset = iter.offset() - suffix.chars().count() as i32;
            let new_iter = buffer.iter_at_offset(new_offset);
            buffer.place_cursor(&new_iter);
        }
        buffer.end_user_action();
    }

    impl WidgetImpl for TextBlockWidget {}

    impl TextBlockWidget {
        pub fn init_data(&self, data: TextBlock) {
            self.data.replace(data.clone());

            if data.content.starts_with("[IMAGE]") {
                let path = data.content.trim_start_matches("[IMAGE]");
                self.picture.set_filename(Some(path));
                self.stack.set_visible_child(&self.picture);
            } else if data.content.starts_with("[FILE]") {
                let path = data.content.trim_start_matches("[FILE]").to_string();
                let file_name = std::path::Path::new(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                self.file_btn.set_label(&format!("📎 Open {}", file_name));
                self.file_btn.connect_clicked(move |_| {
                    let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                });
                self.stack.set_visible_child(&self.file_box);
            } else if data.content.starts_with("[TIMER]") {
                self.stack.set_visible_child(&self.timer_box);
            } else if data.content.starts_with("[CODE]") {
                let code = data.content.trim_start_matches("[CODE]\n");
                self.code_view.buffer().set_text(code);
                self.stack.set_visible_child(&self.code_view);
            } else if data.content.starts_with("[CHECKLIST]") {
                self.render_checklist();
                self.stack.set_visible_child(&self.checklist_box);
            } else {
                self.text_view.buffer().set_text(&data.content);
                self.apply_markdown();
                self.stack.set_visible_child(&self.text_view);
            }

            self.obj()
                .set_size_request(data.width as i32, data.height as i32);
        }

        pub fn render_checklist(&self) {
            while let Some(child) = self.checklist_box.first_child() {
                self.checklist_box.remove(&child);
            }

            let content = self.data.borrow().content.clone();
            let json_str = content.trim_start_matches("[CHECKLIST]").trim();
            let items: Vec<ChecklistItem> = serde_json::from_str(json_str).unwrap_or_default();

            for (idx, item) in items.iter().enumerate() {
                let check = gtk::CheckButton::builder()
                    .label(&item.text)
                    .active(item.checked)
                    .build();

                if item.checked {
                    check.add_css_class("strikethrough");
                }

                let obj = self.obj();
                check.connect_toggled(glib::clone!(
                    #[weak]
                    obj,
                    move |btn| {
                        let is_active = btn.is_active();
                        if is_active {
                            btn.add_css_class("strikethrough");
                        } else {
                            btn.remove_css_class("strikethrough");
                        }
                        obj.imp().update_checklist_item(idx, is_active);
                    }
                ));

                self.checklist_box.append(&check);
            }

            // Add new item entry
            let entry = gtk::Entry::builder()
                .placeholder_text("Add item...")
                .has_frame(false)
                .build();

            let obj = self.obj();
            entry.connect_activate(glib::clone!(
                #[weak]
                obj,
                move |ent| {
                    let text = ent.text().to_string();
                    if !text.is_empty() {
                        obj.imp().add_checklist_item(text);
                    }
                }
            ));

            self.checklist_box.append(&entry);
        }

        fn update_checklist_item(&self, idx: usize, checked: bool) {
            let content = self.data.borrow().content.clone();
            let json_str = content.trim_start_matches("[CHECKLIST]").trim();
            if let Ok(mut items) = serde_json::from_str::<Vec<ChecklistItem>>(json_str) {
                if let Some(item) = items.get_mut(idx) {
                    item.checked = checked;
                    let new_json = serde_json::to_string(&items).unwrap_or_default();
                    self.data.borrow_mut().content = format!("[CHECKLIST] {}", new_json);
                    self.save_data_no_sync();
                }
            }
        }

        fn add_checklist_item(&self, text: String) {
            let content = self.data.borrow().content.clone();
            let json_str = content.trim_start_matches("[CHECKLIST]").trim();
            let mut items: Vec<ChecklistItem> = serde_json::from_str(json_str).unwrap_or_default();

            items.push(ChecklistItem {
                text,
                checked: false,
            });
            let new_json = serde_json::to_string(&items).unwrap_or_default();
            self.data.borrow_mut().content = format!("[CHECKLIST] {}", new_json);
            self.save_data_no_sync();
            self.render_checklist();
        }

        fn save_data_no_sync(&self) {
            if let Some(db) = DB.lock().unwrap().as_ref() {
                let _ = db.upsert_block(&self.data.borrow());
            }
        }

        pub fn save_data(&self) {
            let mut data = self.data.borrow_mut();
            if data.content.starts_with("[IMAGE]")
                || data.content.starts_with("[CHECKLIST]")
                || data.content.starts_with("[FILE]")
                || data.content.starts_with("[TIMER]")
            {
                // Do nothing, content is updated elsewhere
            } else if data.content.starts_with("[CODE]") {
                let buffer = self.code_view.buffer();
                data.content = format!(
                    "[CODE]\n{}",
                    buffer.text(&buffer.start_iter(), &buffer.end_iter(), false)
                );
            } else {
                let buffer = self.text_view.buffer();
                data.content = buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), false)
                    .to_string();
            }

            if let Some(db) = DB.lock().unwrap().as_ref() {
                let _ = db.upsert_block(&data);
            }
        }
    }
}
