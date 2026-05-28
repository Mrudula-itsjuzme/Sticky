import re

with open("src/main.rs", "r") as f:
    content = f.read()

# 1. Remove the tray logic from where it is now.
start_idx = content.find("// Use std mpsc: tray thread sends a")
end_idx = content.find("app.run()")

if start_idx == -1 or end_idx == -1:
    print("Could not find bounds")
    exit(1)

tray_code = content[start_idx:end_idx].strip()
content = content[:start_idx] + content[end_idx:]

# 2. Insert the tray code inside connect_startup
startup_end_pattern = r'app\.set_accels_for_action\("app\.search", &\["<Control><Shift>f"\]\);\n\s*\}\);'
startup_end_repl = f'''app.set_accels_for_action("app.search", &["<Control><Shift>f"]);
        
        {tray_code}
    }});'''
content = re.sub(startup_end_pattern, startup_end_repl, content)

with open("src/main.rs", "w") as f:
    f.write(content)
