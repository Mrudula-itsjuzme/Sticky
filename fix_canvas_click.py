import re

with open("src/canvas.rs", "r") as f:
    content = f.read()

# Make the label not interactable
label_pattern = r'self\.empty_label\.set_valign\(gtk::Align::Center\);'
label_repl = '''self.empty_label.set_valign(gtk::Align::Center);
            self.empty_label.set_can_target(false);'''
content = re.sub(label_pattern, label_repl, content)

with open("src/canvas.rs", "w") as f:
    f.write(content)
