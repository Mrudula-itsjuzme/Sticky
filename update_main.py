import re

with open("src/main.rs", "r") as f:
    content = f.read()

# Make application handle command line
flags_pattern = r'\.application_id\("com.mrudula.sticky"\)\n\s*\.build\(\);'
flags_repl = '''.application_id("com.mrudula.sticky")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();'''
content = re.sub(flags_pattern, flags_repl, content)

with open("src/main.rs", "w") as f:
    f.write(content)
