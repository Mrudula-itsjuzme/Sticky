import re

with open("src/main.rs", "r") as f:
    content = f.read()

# Replace connect_activate with connect_command_line
old_activate_pattern = r'app\.connect_activate\(\|app\| \{\n\s*println!\("App activated"\);(.*?)\n\s*\}\);\n\n\s*app\.run\(\);'
match = re.search(old_activate_pattern, content, re.DOTALL)
if not match:
    print("Could not find connect_activate")
    exit(1)

inner_logic = match.group(1)

new_logic = '''
    app.connect_command_line(|app, cmdline| {
        println!("App command-line received");
        let args = cmdline.arguments();
        let mut force_new = false;
        let mut background = false;
        let mut quit = false;

        for arg in args.iter().skip(1) {
            let arg_str = arg.to_string_lossy();
            match arg_str.as_ref() {
                "--new-note" => force_new = true,
                "--background" => background = true,
                "--quit" => quit = true,
                _ => {}
            }
        }

        if quit {
            app.quit();
            return 0;
        }

        if force_new {
            portals::create_new_note(app);
            return 0;
        }

        let db_arc_opt = DB.lock().unwrap().as_ref().cloned();
        if let Some(db_arc) = db_arc_opt {
            if let Ok(notes) = db_arc.get_notes() {
                println!("Found {} notes in DB", notes.len());
                if notes.is_empty() && !background {
                    println!("No notes found, creating a default one...");
                    let _ = db_arc.create_note(100, 100, "#FFE66D");
                }

                let mut active_windows = Vec::new();
                for note in db_arc.get_notes().unwrap_or_default() {
                    let mut found = false;
                    for w in app.windows() {
                        if let Some(sticky) = w.downcast_ref::<window::StickyWindow>() {
                            if let Some(n) = sticky.imp().note.borrow().as_ref() {
                                if n.id == note.id {
                                    sticky.present();
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !found {
                        let window = window::StickyWindow::new(app, note.clone());
                        window.present();
                        active_windows.push(window);
                    }
                }
                
                WINDOWS.with(|w| {
                    let mut current = w.borrow_mut();
                    for win in active_windows {
                        current.push(win);
                    }
                });
            }
        }

        // Initialize tray only if not already initialized
        // Wait, tray logic needs to only happen once.
        // Actually, command_line runs in the primary instance!
        // We can just initialize tray in connect_startup.
        0
    });

    app.run();
'''

content = content[:match.start()] + new_logic + content[match.end():]

# We need to move the tray logic to connect_startup.
# Let's extract the tray logic from the old inner_logic.
tray_pattern = r'// --- System Tray Icon ---.*'
tray_match = re.search(tray_pattern, inner_logic, re.DOTALL)
if tray_match:
    tray_logic = tray_match.group(0)
    
    # insert tray logic at end of connect_startup
    startup_end_pattern = r'app\.set_accels_for_action\("app\.search", &\["<Control><Shift>f"\]\);\n\s*\}\);'
    startup_end_repl = f'''app.set_accels_for_action("app.search", &["<Control><Shift>f"]);
        
        {tray_logic}
    }});'''
    content = re.sub(startup_end_pattern, startup_end_repl, content)

with open("src/main.rs", "w") as f:
    f.write(content)

