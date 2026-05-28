import re

with open("src/canvas.rs", "r") as f:
    content = f.read()

# Replace struct Canvas with empty state label
struct_pattern = r'pub struct Canvas \{\n\s*pub note_id: Cell<i64>,\n\s*pub whiteboard_mode: Cell<bool>,\n\s*pub linking_state: RefCell<Option<i64>>,\n\s*\}'
struct_repl = '''pub struct Canvas {
        pub note_id: Cell<i64>,
        pub whiteboard_mode: Cell<bool>,
        pub linking_state: RefCell<Option<i64>>,
        pub empty_label: gtk::Label,
    }'''
content = re.sub(struct_pattern, struct_repl, content)

# update constructed to init empty_label
constructed_pattern = r'fn constructed\(&self\) \{\n\s*self.parent_constructed\(\);\n\s*let obj = self.obj\(\);'
constructed_repl = '''fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            
            self.empty_label.set_label("Double-click to write...");
            self.empty_label.add_css_class("sticky-empty-state");
            self.empty_label.set_halign(gtk::Align::Center);
            self.empty_label.set_valign(gtk::Align::Center);
            obj.put(&self.empty_label, 16.0, 16.0);
'''
content = re.sub(constructed_pattern, constructed_repl, content)

# update add_block to hide it
add_block_pattern = r'fn add_block\(&self, data: TextBlock\) \{\n\s*let block = TextBlockWidget::new\(data.clone\(\)\);\n\s*self.put\(&block, data.x, data.y\);\n\s*\}'
add_block_repl = '''fn add_block(&self, data: TextBlock) {
        let block = TextBlockWidget::new(data.clone());
        self.put(&block, data.x, data.y);
        self.imp().empty_label.set_visible(false);
    }'''
content = re.sub(add_block_pattern, add_block_repl, content)

# update remove_block to show it if empty
remove_block_pattern = r'self.remove\(&block\);\n\s*\}'
remove_block_repl = '''self.remove(block);
        
        let mut has_blocks = false;
        let mut child = self.first_child();
        while let Some(widget) = child {
            if widget.downcast_ref::<TextBlockWidget>().is_some() {
                has_blocks = true;
                break;
            }
            child = widget.next_sibling();
        }
        self.imp().empty_label.set_visible(!has_blocks);
    }'''
content = re.sub(remove_block_pattern, remove_block_repl, content)

with open("src/canvas.rs", "w") as f:
    f.write(content)
