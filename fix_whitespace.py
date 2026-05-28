with open("src/window.rs", "r") as f:
    lines = f.readlines()

with open("src/window.rs", "w") as f:
    for line in lines:
        f.write(line.rstrip() + "\n")
