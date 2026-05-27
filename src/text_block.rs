use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use crate::db::TextBlock;
use crate::DB;

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
}

mod imp {
    use super::*;
    use std::cell::RefCell;

    pub struct TextBlockWidget {
        pub data: RefCell<TextBlock>,
        pub text_view: gtk::TextView,
    }

    impl Default for TextBlockWidget {
        fn default() -> Self {
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
                text_view: gtk::TextView::builder()
                    .wrap_mode(gtk::WrapMode::Word)
                    .accepts_tab(false)
                    .build(),
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
            frame.set_child(Some(&self.text_view));
            overlay.set_child(Some(&frame));

            let delete_btn = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .css_classes(["delete-btn", "flat"])
                .halign(gtk::Align::End)
                .valign(gtk::Align::Start)
                .build();
            
            delete_btn.connect_clicked(glib::clone!(#[weak] obj, move |_| {
                if let Some(canvas) = obj.parent().and_downcast::<crate::canvas::Canvas>() {
                    canvas.remove_block(&obj);
                }
            }));
            
            overlay.add_overlay(&delete_btn);
            overlay.set_parent(&*obj);

            let drag = gtk::GestureDrag::new();
            obj.add_controller(drag.clone());

            drag.connect_drag_update(glib::clone!(#[weak] obj, move |drag, offset_x, offset_y| {
                if let Some(parent) = obj.parent().and_downcast::<gtk::Fixed>() {
                    let data = obj.imp().data.borrow();
                    let new_x = data.x + offset_x;
                    let new_y = data.y + offset_y;
                    parent.move_(&obj, new_x, new_y);
                    drag.set_state(gtk::EventSequenceState::Claimed);
                }
            }));

            drag.connect_drag_end(glib::clone!(#[weak] obj, move |_, offset_x, offset_y| {
                {
                    let mut data = obj.imp().data.borrow_mut();
                    data.x += offset_x;
                    data.y += offset_y;
                }
                obj.imp().save_data();
            }));

            let buffer = self.text_view.buffer();
            buffer.connect_changed(glib::clone!(#[weak] obj, move |_| {
                obj.imp().save_data();
            }));
            
            // Initial styling is now handled by style.css
        }

        fn dispose(&self) {
            if let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for TextBlockWidget {}

    impl TextBlockWidget {
        pub fn init_data(&self, data: TextBlock) {
            self.data.replace(data.clone());
            self.text_view.buffer().set_text(&data.content);
            self.obj().set_size_request(data.width as i32, data.height as i32);
        }

        pub fn save_data(&self) {
            let mut data = self.data.borrow_mut();
            let buffer = self.text_view.buffer();
            data.content = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
            
            if let Some(db) = DB.lock().unwrap().as_ref() {
                let _ = db.upsert_block(&data);
            }
        }
    }
}
