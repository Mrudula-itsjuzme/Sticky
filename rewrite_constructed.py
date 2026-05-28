import re

with open("src/window.rs", "r") as f:
    content = f.read()

with open("new_constructed.rs", "r") as f:
    new_constructed = f.read()

# We want to replace everything from `fn constructed(&self) {` to the matching closing brace.
start_idx = content.find("fn constructed(&self) {")
if start_idx == -1:
    print("Could not find constructed")
    exit(1)

# Find the matching closing brace for constructed
brace_count = 0
end_idx = -1
for i in range(start_idx, len(content)):
    if content[i] == '{':
        brace_count += 1
    elif content[i] == '}':
        brace_count -= 1
        if brace_count == 0:
            end_idx = i + 1
            break

if end_idx == -1:
    print("Could not find end of constructed")
    exit(1)

new_content = content[:start_idx] + new_constructed.strip() + "\n" + content[end_idx:]

with open("src/window.rs", "w") as f:
    f.write(new_content)
    print("Success")
