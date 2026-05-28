use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib, gdk};
use crate::db::Note;
use crate::canvas::Canvas;
use crate::DB;
use crate::portals;

glib::wrapper! {
    pub struct StickyWindow(ObjectSubclass<imp::StickyWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl StickyWindow {
    pub fn new(app: &adw::Application, note: Note) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("title", "Sticky")
            .property("decorated", false)
            .property("default-width", note.width)
            .property("default-height", note.height)
            .build();
        
        window.imp().init_note(note);
        window
    }
}

mod imp {
    use super::*;
    use std::cell::RefCell;
    use std::process::{Command, Child};

    #[derive(Default)]
    pub struct StickyWindow {
        pub note: RefCell<Option<Note>>,
        pub recording_process: RefCell<Option<Child>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StickyWindow {
        const NAME: &'static str = "StickyWindow";
        type Type = super::StickyWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for StickyWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.add_css_class("sticky-note");
            if let Some(note) = obj.imp().note.borrow().as_ref() {
                obj.add_css_class(&format!("note-{}", note.id));
            }
            
            let header = adw::HeaderBar::builder()
                .show_end_title_buttons(false)
                .show_start_title_buttons(false)
                .build();
            
            let color_button = gtk::Button::builder()
                .icon_name("color-select-symbolic")
                .tooltip_text("Pick Color")
                .build();
            
            color_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                glib::spawn_future_local(async move {
                    if let Ok(Some(hex)) = portals::pick_color().await {
                        obj.imp().update_color(hex);
                    }
                });
            }));
            header.pack_start(&color_button);

            let new_note_button = gtk::Button::builder()
                .icon_name("list-add-symbolic")
                .tooltip_text("New Note")
                .build();
            new_note_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(app) = obj.application().and_downcast::<adw::Application>() {
                    portals::create_new_note(&app);
                }
            }));
            header.pack_start(&new_note_button);

            let share_button = gtk::Button::builder()
                .icon_name("edit-copy-symbolic")
                .tooltip_text("Copy All Text")
                .build();
            share_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                    if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                        let text = canvas.get_all_text();
                        if let Some(display) = gtk::gdk::Display::default() {
                            display.clipboard().set_text(&text);
                        }
                    }
                }
            }));
            header.pack_start(&share_button);

            let checklist_button = gtk::Button::builder()
                .icon_name("view-list-symbolic")
                .tooltip_text("Add Checklist")
                .build();
            checklist_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                    if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                        canvas.create_block_with_content(20.0, 100.0, "[CHECKLIST] []".to_string());
                    }
                }
            }));
            header.pack_start(&checklist_button);

            let code_button = gtk::Button::builder()
                .icon_name("text-editor-symbolic")
                .tooltip_text("Add Code Snippet")
                .build();
            code_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                    if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                        canvas.create_block_with_content(20.0, 120.0, "[CODE]\n// Write code here...".to_string());
                    }
                }
            }));
            header.pack_start(&code_button);

            let timer_button = gtk::Button::builder()
                .icon_name("alarm-symbolic")
                .tooltip_text("Pomodoro Timer")
                .build();
            timer_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                    if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                        canvas.create_block_with_content(20.0, 140.0, "[TIMER]".to_string());
                    }
                }
            }));
            header.pack_start(&timer_button);

            let mic_button = gtk::Button::builder()
                .icon_name("audio-input-microphone-symbolic")
                .tooltip_text("Capture Audio")
                .build();

            mic_button.connect_clicked(glib::clone!(#[weak] obj, move |btn| {
                let mut proc_opt = obj.imp().recording_process.borrow_mut();
                if proc_opt.is_none() {
                    // Start recording
                    btn.add_css_class("recording-active");
                    let child = Command::new("arecord")
                        .args(["-f", "S16_LE", "-r", "16000", "/tmp/meeting.wav"])
                        .spawn();
                    
                    if let Ok(child) = child {
                        *proc_opt = Some(child);
                    }
                } else {
                    // Stop recording
                    btn.remove_css_class("recording-active");
                    if let Some(mut child) = proc_opt.take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    
                    if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                        if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                            let canvas = canvas.clone();
                            glib::MainContext::default().spawn_local(async move {
                                let result = crate::TOKIO_RT.spawn(async move {
                                    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
                                    if api_key.is_empty() {
                                        return Err("🎙️ Transcription failed: OPENAI_API_KEY not set.".to_string());
                                    }
                                    
                                    let file_bytes = std::fs::read("/tmp/meeting.wav").unwrap_or_default();
                                    if file_bytes.is_empty() { return Err("Failed to read audio file".to_string()); }
                                    
                                    let part = reqwest::multipart::Part::bytes(file_bytes)
                                        .file_name("meeting.wav")
                                        .mime_str("audio/wav")
                                        .unwrap();
                                    let form = reqwest::multipart::Form::new()
                                        .text("model", "whisper-1")
                                        .part("file", part);
                                        
                                    let client = reqwest::Client::new();
                                    let res = client.post("https://api.openai.com/v1/audio/transcriptions")
                                        .bearer_auth(&api_key)
                                        .multipart(form)
                                        .send()
                                        .await;
                                        
                                    let mut transcription = String::new();
                                    if let Ok(res) = res {
                                        if let Ok(json) = res.json::<serde_json::Value>().await {
                                            if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                                                transcription = text.to_string();
                                            }
                                        }
                                    }
                                    
                                    if !transcription.is_empty() {
                                        let prompt = format!("Summarize the meeting and extract action items. Return a JSON object with 'summary' (string) and 'action_items' (array of strings):\n\n{}", transcription);
                                        let payload = serde_json::json!({
                                            "model": "gpt-4o-mini",
                                            "response_format": { "type": "json_object" },
                                            "messages": [{"role": "user", "content": prompt}]
                                        });
                                        
                                        let sum_res = client.post("https://api.openai.com/v1/chat/completions")
                                            .bearer_auth(&api_key)
                                            .json(&payload)
                                            .send()
                                            .await;
                                            
                                        if let Ok(r) = sum_res {
                                            if let Ok(json) = r.json::<serde_json::Value>().await {
                                                if let Some(msg) = json["choices"][0]["message"]["content"].as_str() {
                                                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg) {
                                                        let summary = parsed["summary"].as_str().unwrap_or("").to_string();
                                                        let action_items: Vec<String> = parsed["action_items"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                                        return Ok((summary, action_items));
                                                    }
                                                }
                                            }
                                        }
                                        Err("Failed to parse summary API response".to_string())
                                    } else {
                                        Err("Failed to transcribe audio. Ensure microphone is working.".to_string())
                                    }
                                }).await.unwrap_or_else(|_| Err("Async task panicked".to_string()));
                                
                                match result {
                                    Ok((summary, action_items)) => {
                                        canvas.create_block_with_content(20.0, 40.0, format!("🎙️ Meeting Summary:\n\n{}", summary));
                                        
                                        if !action_items.is_empty() {
                                            use crate::text_block::ChecklistItem;
                                            let items: Vec<ChecklistItem> = action_items.into_iter().map(|t| ChecklistItem { text: t, checked: false }).collect();
                                            if let Ok(json_str) = serde_json::to_string(&items) {
                                                canvas.create_block_with_content(20.0, 200.0, format!("[CHECKLIST] {}", json_str));
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        canvas.create_block_with_content(20.0, 40.0, err);
                                    }
                                }
                            });
                        }
                    }
                }
            }));
            header.pack_start(&mic_button);

            let trash_button = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .tooltip_text("Delete Note")
                .css_classes(["flat"])
                .build();
            trash_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                    content_box.add_css_class("peel-out");
                }
                let obj_weak = obj.downgrade();
                glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                    if let Some(obj) = obj_weak.upgrade() {
                        if let Some(note) = obj.imp().note.borrow().as_ref() {
                            if let Some(db) = DB.lock().unwrap().as_ref() {
                                let _ = db.delete_note(note.id);
                            }
                        }
                        obj.close();
                    }
                    glib::ControlFlow::Break
                });
            }));
            header.pack_end(&trash_button);

            let close_button = gtk::Button::builder()
                .icon_name("window-close-symbolic")
                .tooltip_text("Hide Note")
                .css_classes(["flat"])
                .build();
            close_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                obj.close(); // Only hide/destroy window, remains in DB
            }));
            header.pack_end(&close_button);

            let always_on_top_button = gtk::ToggleButton::builder()
                .icon_name("pin-symbolic")
                .tooltip_text("Always on Top")
                .css_classes(["flat"])
                .build();
            
            if let Some(note) = obj.imp().note.borrow().as_ref() {
                always_on_top_button.set_active(note.always_on_top);
            }

            always_on_top_button.connect_toggled(glib::clone!(#[weak] obj, move |btn| {
                let active = btn.is_active();
                let note_id = {
                    obj.imp().note.borrow().as_ref().map(|n| n.id)
                };
                
                if let Some(id) = note_id {
                    if let Some(db) = DB.lock().unwrap().as_ref() {
                        let _ = db.update_note_always_on_top(id, active);
                    }
                }
            }));
            header.pack_end(&always_on_top_button);

            let expand_button = gtk::Button::builder()
                .icon_name("view-fullscreen-symbolic")
                .tooltip_text("Expand to Whiteboard")
                .css_classes(["flat"])
                .build();
            expand_button.connect_clicked(glib::clone!(#[weak] obj, move |btn| {
                if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                    if obj.is_maximized() {
                        // --- Exit whiteboard mode ---
                        obj.unmaximize();
                        btn.set_icon_name("view-fullscreen-symbolic");
                        btn.set_tooltip_text(Some("Expand to Whiteboard"));
                        obj.remove_css_class("whiteboard-mode");
                        // Unwrap ScrolledWindow, put canvas back directly
                        if let Some(scroll) = content_box.last_child().and_downcast::<gtk::ScrolledWindow>() {
                            if let Some(canvas) = scroll.child().and_downcast::<Canvas>() {
                                canvas.set_whiteboard_mode(false);
                                scroll.set_child(gtk::Widget::NONE);
                                content_box.remove(&scroll);
                                content_box.append(&canvas);
                            }
                        }
                    } else {
                        // --- Enter whiteboard mode ---
                        obj.maximize();
                        btn.set_icon_name("view-restore-symbolic");
                        btn.set_tooltip_text(Some("Exit Whiteboard"));
                        obj.add_css_class("whiteboard-mode");
                        // Wrap canvas in ScrolledWindow for infinite-canvas feel
                        if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                            canvas.set_whiteboard_mode(true);
                            content_box.remove(&canvas);
                            let scroll = gtk::ScrolledWindow::builder()
                                .hexpand(true)
                                .vexpand(true)
                                .child(&canvas)
                                .build();
                            content_box.append(&scroll);
                        }
                    }
                }
            }));
            header.pack_end(&expand_button);

            let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            content_box.append(&header);

            let canvas = Canvas::new();
            content_box.append(&canvas);
            
            obj.set_content(Some(&content_box));

            // Drag support for borderless window
            let drag = gtk::GestureClick::new();
            header.add_controller(drag.clone());
            
            drag.connect_pressed(glib::clone!(#[weak] obj, move |gesture, n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                
                if n == 2 { // Double click to roll-up
                    if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                        if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                            let is_visible = canvas.is_visible();
                            canvas.set_visible(!is_visible);
                            
                            if is_visible {
                                // Roll up
                                obj.set_size_request(-1, -1);
                                obj.set_default_size(-1, -1);
                            } else {
                                // Unroll
                                if let Some(note) = obj.imp().note.borrow().as_ref() {
                                    obj.set_size_request(note.width, note.height);
                                    obj.set_default_size(note.width, note.height);
                                }
                            }
                        }
                    }
                    return;
                }

                if let Some(surface) = obj.surface() {
                    if let Some(toplevel) = surface.downcast_ref::<gdk::Toplevel>() {
                        if let Some(device) = gesture.device() {
                            toplevel.begin_move(&device, 1, x, y, gesture.current_event_time());
                        }
                    }
                }
            }));
            
            // Command Palette (Ctrl+K)
            let key_ctrl = gtk::EventControllerKey::new();
            let obj_weak = obj.downgrade();
            let header_clone = header.clone();
            key_ctrl.connect_key_pressed(move |_, keyval, _keycode, state| {
                if let Some(obj) = obj_weak.upgrade() {
                    if state.contains(gdk::ModifierType::CONTROL_MASK) && keyval == gdk::Key::k {
                        obj.imp().show_command_palette(header_clone.upcast_ref::<gtk::Widget>());
                        return glib::Propagation::Stop;
                    }
                }
                glib::Propagation::Proceed
            });
            obj.add_controller(key_ctrl);
            
            obj.connect_default_width_notify(|obj| {
                obj.imp().save_state();
            });
            obj.connect_default_height_notify(|obj| {
                obj.imp().save_state();
            });
        }
    }

    impl WidgetImpl for StickyWindow {}
    impl WindowImpl for StickyWindow {}
    impl ApplicationWindowImpl for StickyWindow {}
    impl AdwApplicationWindowImpl for StickyWindow {}

    impl StickyWindow {
        pub fn init_note(&self, note: Note) {
            self.note.replace(Some(note.clone()));
            
            let obj = self.obj();
            obj.add_css_class(&format!("note-{}", note.id));
            self.apply_color(note.id, &note.color);
            
            if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                    canvas.load_note(note.id);
                }
            }
        }

        pub fn update_color(&self, hex: String) {
            let mut note_opt = self.note.borrow_mut();
            if let Some(note) = note_opt.as_mut() {
                note.color = hex.clone();
                self.apply_color(note.id, &hex);
                if let Some(db) = DB.lock().unwrap().as_ref() {
                    let _ = db.update_note_color(note.id, &hex);
                }
            }
        }

        pub fn show_command_palette(&self, anchor: &gtk::Widget) {
            let popover = gtk::Popover::builder()
                .position(gtk::PositionType::Bottom)
                .autohide(true)
                .has_arrow(false)
                .build();
            
            popover.set_parent(anchor);
            popover.add_css_class("command-palette");

            let list_box = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::None)
                .build();

            let actions = vec![
                ("📝 Add Checklist", "[CHECKLIST] []"),
                ("💻 Add Code Snippet", "[CODE]\n// type here"),
                ("⏱️ Add Timer", "[TIMER]"),
                ("🔴 Red Background", "#FF6B6B"),
                ("🔵 Blue Background", "#4ECDC4"),
                ("🟡 Yellow Background", "#FFE66D"),
            ];

            for (label, action) in actions {
                let btn = gtk::Button::with_label(label);
                btn.add_css_class("flat");
                let obj = self.obj().clone();
                let pop_ref = popover.clone();
                btn.connect_clicked(move |_| {
                    pop_ref.popdown();
                    if action.starts_with('#') {
                        obj.imp().update_color(action.to_string());
                    } else if let Some(content_box) = obj.content().and_downcast::<gtk::Box>() {
                        if let Some(canvas) = content_box.last_child().and_downcast::<Canvas>() {
                            canvas.create_block_with_content(50.0, 50.0, action.to_string());
                        }
                    }
                });
                list_box.append(&btn);
            }

            popover.set_child(Some(&list_box));
            popover.popup();
        }

        fn apply_color(&self, id: i64, hex: &str) {
            let css = format!(
                ".note-{} {{ background-color: {}; }}",
                id, hex
            );
            let provider = gtk::CssProvider::new();
            provider.load_from_data(&css);
            
            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().expect("Could not connect to a display."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
            );
        }

        pub fn save_state(&self) {
            if let Some(note) = self.note.borrow().as_ref() {
                if let Some(db) = DB.lock().unwrap().as_ref() {
                    let (w, h) = self.obj().default_size();
                    let _ = db.update_note_size(note.id, w, h);
                }
            }
        }
    }
}
