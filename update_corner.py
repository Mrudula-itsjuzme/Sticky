import re

with open("src/window.rs", "r") as f:
    content = f.read()

corner_pattern = r'\.css_classes\(\["sticky-folded-corner"\]\)\n\s*\.valign\(gtk::Align::End\)'
corner_repl = '''.css_classes(["sticky-folded-corner"])
                .valign(gtk::Align::Start)'''
content = re.sub(corner_pattern, corner_repl, content)

with open("src/window.rs", "w") as f:
    f.write(content)
