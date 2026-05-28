import re

with open("src/canvas.rs", "r") as f:
    content = f.read()

pattern = r'            // Draw whiteboard dot-grid background\n\s*if self\.whiteboard_mode\.get\(\) \{\n\s*let spacing = 28\.0f64;\n\s*let dot_r = 1\.5f32;\n\n\s*let cr = snapshot\.append_cairo\(&gtk::graphene::Rect::new\(\n\s*0\.0,\n\s*0\.0,\n\s*width as f32,\n\s*height as f32,\n\s*\)\);\n\s*// Whiteboard white fill\n\s*cr\.set_source_rgba\(0\.98, 0\.98, 1\.0, 1\.0\);\n\s*cr\.paint\(\)\.ok\(\);\n\s*// Soft dots'

repl = '''            // Draw whiteboard dot-grid background
            if self.whiteboard_mode.get() {
                let spacing = 28.0f64;
                let dot_r = 1.5f32;

                // Whiteboard white fill
                cr.set_source_rgba(0.98, 0.98, 1.0, 1.0);
                cr.paint().ok();
                // Soft dots'''

if re.search(pattern, content):
    content = re.sub(pattern, repl, content)
else:
    print("Pattern not found!")

with open("src/canvas.rs", "w") as f:
    f.write(content)
