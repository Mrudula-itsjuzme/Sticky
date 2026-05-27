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
            .property("title", "Antigrav")
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

    #[derive(Default)]
    pub struct StickyWindow {
        pub note: RefCell<Option<Note>>,
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

            let close_button = gtk::Button::builder()
                .icon_name("window-close-symbolic")
                .css_classes(["flat"])
                .build();
            close_button.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(note) = obj.imp().note.borrow().as_ref() {
                    if let Some(db) = DB.lock().unwrap().as_ref() {
                        let _ = db.delete_note(note.id);
                    }
                }
                obj.close();
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

            let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            content_box.append(&header);

            let canvas = Canvas::new();
            content_box.append(&canvas);
            
            obj.set_content(Some(&content_box));

            // Drag support for borderless window
            let drag = gtk::GestureClick::new();
            header.add_controller(drag.clone());
            
            drag.connect_pressed(glib::clone!(#[weak] obj, move |gesture, _, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(surface) = obj.surface() {
                    if let Some(toplevel) = surface.downcast_ref::<gdk::Toplevel>() {
                        if let Some(device) = gesture.device() {
                            toplevel.begin_move(&device, 1, x, y, gesture.current_event_time());
                        }
                    }
                }
            }));
            
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
