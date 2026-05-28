import re

with open("src/window.rs", "r") as f:
    content = f.read()

# Replace default_width_notify logic
notify_pattern = r'obj\.connect_default_width_notify\(glib::clone!\(\n\s*#\[weak\]\n\s*obj,\n\s*move \|_\| \{\n\s*obj\.imp\(\)\.save_state\(\);\n\s*\}\n\s*\)\);'
notify_repl = '''obj.connect_default_width_notify(glib::clone!(
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
                            obj_clone.imp().save_state();
                        }
                    ));
                }
            ));'''
content = re.sub(notify_pattern, notify_repl, content)

# Add save_timer
struct_pattern = r'pub struct StickyWindow \{\n\s*pub note: RefCell<Option<Note>>,'
struct_repl = '''pub struct StickyWindow {
        pub note: RefCell<Option<Note>>,
        pub save_timer: RefCell<Option<glib::SourceId>>,'''
content = re.sub(struct_pattern, struct_repl, content)

init_pattern = r'note: RefCell::new\(None\),'
init_repl = '''note: RefCell::new(None),
                save_timer: RefCell::new(None),'''
content = re.sub(init_pattern, init_repl, content)

with open("src/window.rs", "w") as f:
    f.write(content)
