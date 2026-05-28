import re

with open("src/text_block.rs", "r") as f:
    content = f.read()

# Add save_timer
struct_pattern = r'pub struct TextBlockWidget \{\n\s*pub data: RefCell<TextBlock>,'
struct_repl = '''pub struct TextBlockWidget {
        pub data: RefCell<TextBlock>,
        pub save_timer: RefCell<Option<glib::SourceId>>,'''
content = re.sub(struct_pattern, struct_repl, content)

# Initialize save_timer
init_pattern = r'data: RefCell::new\(TextBlock \{'
init_repl = '''save_timer: RefCell::new(None),\n                data: RefCell::new(TextBlock {'''
content = re.sub(init_pattern, init_repl, content)

# Replace the save_data calls with a timeout
drag_end_pattern = r'obj\.imp\(\)\.save_data\(\);\n\s*\}\n\s*\)\);'
drag_end_repl = '''let obj_clone = obj.clone();
                    let mut timer = obj.imp().save_timer.borrow_mut();
                    if let Some(t) = timer.take() {
                        t.remove();
                    }
                    *timer = Some(glib::timeout_add_local_once(
                        std::time::Duration::from_millis(500),
                        move || {
                            obj_clone.imp().save_data();
                        }
                    ));
                }
            ));'''
content = re.sub(drag_end_pattern, drag_end_repl, content)

text_changed_pattern = r'buffer\.connect_changed\(glib::clone!\(\n\s*#\[weak\]\n\s*obj,\n\s*move \|_\| \{\n\s*obj\.imp\(\)\.save_data\(\);\n\s*\}\n\s*\)\);'
text_changed_repl = '''buffer.connect_changed(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    let obj_clone = obj.clone();
                    let mut timer = obj.imp().save_timer.borrow_mut();
                    if let Some(t) = timer.take() {
                        t.remove();
                    }
                    *timer = Some(glib::timeout_add_local_once(
                        std::time::Duration::from_millis(500),
                        move || {
                            obj_clone.imp().save_data();
                        }
                    ));
                }
            ));'''
content = re.sub(text_changed_pattern, text_changed_repl, content)

with open("src/text_block.rs", "w") as f:
    f.write(content)
