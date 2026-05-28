use crate::db::TextBlock;
use crate::text_block::TextBlockWidget;
use crate::DB;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::Canvas>)
        @extends gtk::Widget, gtk::Fixed,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Canvas {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("hexpand", true)
            .property("vexpand", true)
            .build()
    }

    /// Switch into/out of infinite whiteboard mode.
    /// Returns a ScrolledWindow wrapping self when enabling, None when disabling.
    pub fn set_whiteboard_mode(&self, enabled: bool) {
        self.imp().whiteboard_mode.set(enabled);
        if enabled {
            self.add_css_class("whiteboard");
            // Make the canvas much larger for infinite feel
            self.set_size_request(4000, 4000);
        } else {
            self.remove_css_class("whiteboard");
            self.set_size_request(-1, -1);
        }
        self.queue_draw();
    }

    pub fn is_whiteboard(&self) -> bool {
        self.imp().whiteboard_mode.get()
    }

    pub fn load_note(&self, note_id: i64) {
        self.imp().note_id.set(note_id);
        if let Some(db) = DB.lock().unwrap().as_ref() {
            if let Ok(blocks) = db.get_blocks(note_id) {
                for block in blocks {
                    self.add_block(block);
                }
            }
        }
    }

    fn add_block(&self, data: TextBlock) {
        let block = TextBlockWidget::new(data.clone());
        self.put(&block, data.x, data.y);
        self.imp().empty_label.set_visible(false);
    }

    pub fn remove_block(&self, block: &TextBlockWidget) {
        let id = block.imp().data.borrow().id;
        if let Some(db) = DB.lock().unwrap().as_ref() {
            let _ = db.delete_block(id);
        }
        self.remove(block);
    }

    pub fn get_all_text(&self) -> String {
        let mut text = String::new();
        let mut first = true;

        let mut child = self.first_child();
        while let Some(widget) = child {
            if let Some(block) = widget.downcast_ref::<TextBlockWidget>() {
                let content = block.get_content();
                if !content.starts_with("[IMAGE]") {
                    if !first {
                        text.push_str("\n\n");
                    }
                    text.push_str(&content);
                    first = false;
                }
            }
            child = widget.next_sibling();
        }
        text
    }

    pub fn start_linking(&self, block_id: i64) {
        let mut state = self.imp().linking_state.borrow_mut();
        if let Some(source_id) = *state {
            if source_id != block_id {
                if let Some(db) = DB.lock().unwrap().as_ref() {
                    let _ = db.link_blocks(source_id, block_id);
                    self.queue_draw(); // trigger snapshot to redraw arrows
                }
            }
            *state = None;
        } else {
            *state = Some(block_id);
        }
    }
}

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct Canvas {
        pub note_id: Cell<i64>,
        pub whiteboard_mode: Cell<bool>,
        pub linking_state: RefCell<Option<i64>>,
        pub empty_label: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Canvas {
        const NAME: &'static str = "Canvas";
        type Type = super::Canvas;
        type ParentType = gtk::Fixed;
    }

    impl ObjectImpl for Canvas {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            
            self.empty_label.set_label("Double-click to write...");
            self.empty_label.add_css_class("sticky-empty-state");
            self.empty_label.set_halign(gtk::Align::Center);
            self.empty_label.set_valign(gtk::Align::Center);
            obj.put(&self.empty_label, 16.0, 16.0);


            let click = gtk::GestureClick::new();
            obj.add_controller(click.clone());

            click.connect_pressed(glib::clone!(
                #[weak]
                obj,
                move |_, n, x, y| {
                    if n == 2 {
                        // Double click → create block
                        obj.imp().create_block_at(x, y);
                    }
                }
            ));

            let drop_target =
                gtk::DropTarget::new(gtk::gio::File::static_type(), gtk::gdk::DragAction::COPY);
            let obj_weak = obj.downgrade();
            drop_target.connect_drop(move |_, value, x, y| {
                let Some(obj) = obj_weak.upgrade() else {
                    return false;
                };
                if let Ok(file) = value.get::<gtk::gio::File>() {
                    if let Some(path) = file.path() {
                        let path_str = path.display().to_string();
                        let lower = path_str.to_lowercase();
                        if lower.ends_with(".png")
                            || lower.ends_with(".jpg")
                            || lower.ends_with(".jpeg")
                            || lower.ends_with(".gif")
                        {
                            let content = format!("[IMAGE]{}", path_str);
                            obj.create_block_with_content(x, y, content);
                        } else {
                            let content = format!("[FILE]{}", path_str);
                            obj.create_block_with_content(x, y, content);
                        }
                        return true;
                    }
                }
                false
            });
            obj.add_controller(drop_target);
        }
    }

    impl WidgetImpl for Canvas {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            // Draw whiteboard dot-grid background
            if self.whiteboard_mode.get() {
                let width = self.obj().width() as f64;
                let height = self.obj().height() as f64;
                let spacing = 28.0f64;
                let dot_r = 1.5f32;

                let cr = snapshot.append_cairo(&gtk::graphene::Rect::new(
                    0.0,
                    0.0,
                    width as f32,
                    height as f32,
                ));
                // Whiteboard white fill
                cr.set_source_rgba(0.98, 0.98, 1.0, 1.0);
                cr.paint().ok();
                // Soft dots
                cr.set_source_rgba(0.75, 0.78, 0.88, 0.9);
                let mut y = spacing;
                while y < height {
                    let mut x = spacing;
                    while x < width {
                        cr.arc(x, y, dot_r as f64, 0.0, std::f64::consts::TAU);
                        cr.fill().ok();
                        x += spacing;
                    }
                    y += spacing;
                }
            }

            // Draw connecting links
            if let Some(db) = crate::DB.lock().unwrap().as_ref() {
                if let Ok(links) = db.get_links_for_note(self.note_id.get()) {
                    let width = self.obj().width() as f64;
                    let height = self.obj().height() as f64;
                    let cr = snapshot.append_cairo(&gtk::graphene::Rect::new(
                        0.0,
                        0.0,
                        width as f32,
                        height as f32,
                    ));
                    cr.set_source_rgba(0.2, 0.6, 1.0, 0.8);
                    cr.set_line_width(4.0);

                    for (src_id, tgt_id) in links {
                        let mut src_pt = None;
                        let mut tgt_pt = None;

                        let mut child = self.obj().first_child();
                        while let Some(widget) = child {
                            if let Some(block) = widget.downcast_ref::<TextBlockWidget>() {
                                let id = block.imp().data.borrow().id;
                                let x = block.imp().data.borrow().x
                                    + (block.imp().data.borrow().width / 2.0);
                                let y = block.imp().data.borrow().y
                                    + (block.imp().data.borrow().height / 2.0);
                                if id == src_id {
                                    src_pt = Some((x, y));
                                }
                                if id == tgt_id {
                                    tgt_pt = Some((x, y));
                                }
                            }
                            child = widget.next_sibling();
                        }

                        if let (Some((sx, sy)), Some((tx, ty))) = (src_pt, tgt_pt) {
                            cr.move_to(sx, sy);
                            // Cubic bezier curve for beautiful organic mind-map links
                            cr.curve_to(sx + 50.0, sy, tx - 50.0, ty, tx, ty);
                            cr.stroke().ok();
                        }
                    }
                }
            }

            // Draw children on top
            self.parent_snapshot(snapshot);
        }
    }
    impl FixedImpl for Canvas {}

    impl Canvas {
        fn create_block_at(&self, x: f64, y: f64) {
            self.obj().create_block_with_content(x, y, String::new());
        }
    }
}

impl Canvas {
    pub fn create_block_with_content(&self, x: f64, y: f64, content: String) {
        let note_id = self.imp().note_id.get();
        let data = TextBlock {
            id: 0,
            note_id,
            x,
            y,
            width: 250.0,
            height: 150.0,
            content,
        };

        if let Some(db) = DB.lock().unwrap().as_ref() {
            if let Ok(id) = db.upsert_block(&data) {
                let mut new_data = data;
                new_data.id = id;
                self.add_block(new_data);
            }
        }
    }
}
