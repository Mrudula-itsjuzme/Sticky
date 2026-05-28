import re

with open("src/window.rs", "r") as f:
    content = f.read()

# Replace new
new_fn_pattern = r'let width = if note.width >= 300 { note.width } else { 300 };\s*let height = if note.height >= 200 { note.height } else { 200 };'
new_fn_repl = '''let width = if note.width >= 220 { note.width } else { 360 };
        let height = if note.height >= 180 { note.height } else { 320 };'''
content = re.sub(new_fn_pattern, new_fn_repl, content)

builder_pattern = r'\.property\("default-height", height\)\s*\.property\("visible", true\)'
builder_repl = '''.property("default-height", height)
            .property("width-request", 220)
            .property("height-request", 180)
            .property("visible", true)'''
content = re.sub(builder_pattern, builder_repl, content)

# Now rewrite `constructed`. 
# Let's just find the start of constructed and the end.
