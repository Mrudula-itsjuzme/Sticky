use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use crate::text_block::TextBlockWidget;
use crate::DB;
use crate::db::TextBlock;

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
    }

    pub fn remove_block(&self, block: &TextBlockWidget) {
        let id = block.imp().data.borrow().id;
        if let Some(db) = DB.lock().unwrap().as_ref() {
            let _ = db.delete_block(id);
        }
        self.remove(block);
    }
}

mod imp {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    pub struct Canvas {
        pub note_id: Cell<i64>,
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

            let click = gtk::GestureClick::new();
            obj.add_controller(click.clone());

            click.connect_pressed(glib::clone!(#[weak] obj, move |_, n, x, y| {
                if n == 2 { // Double click
                    obj.imp().create_block_at(x, y);
                }
            }));
        }
    }

    impl WidgetImpl for Canvas {}
    impl FixedImpl for Canvas {}

    impl Canvas {
        fn create_block_at(&self, x: f64, y: f64) {
            let note_id = self.note_id.get();
            let data = TextBlock {
                id: 0,
                note_id,
                x,
                y,
                width: 200.0,
                height: 100.0,
                content: String::new(),
            };
            
            if let Some(db) = DB.lock().unwrap().as_ref() {
                if let Ok(id) = db.upsert_block(&data) {
                    let mut new_data = data;
                    new_data.id = id;
                    self.obj().add_block(new_data);
                }
            }
        }
    }
}
