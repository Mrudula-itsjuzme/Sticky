mod canvas;
mod db;
mod portals;
mod text_block;
mod window;

use adw::prelude::*;
use adw::subclass::prelude::ObjectSubclassIsExt;

use db::Db;
use gtk::{gdk, gio};
use once_cell::sync::Lazy;
use std::sync::{mpsc, Arc, Mutex};
use std::cell::RefCell;

thread_local! {
    pub static WINDOWS: RefCell<Vec<window::StickyWindow>> = RefCell::new(Vec::new());
}

pub static DB: Lazy<Mutex<Option<Arc<Db>>>> = Lazy::new(|| Mutex::new(None));
pub static TOKIO_RT: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("sticky {}", env!("CARGO_PKG_VERSION"));
        return glib::ExitCode::SUCCESS;
    }

    if args.iter().any(|arg| arg == "--install-autostart") {
        portals::install_autostart();
        println!("Autostart entry installed.");
        return glib::ExitCode::SUCCESS;
    }

    env_logger::init();

    let app = adw::Application::builder()
        .application_id("com.mrudula.sticky")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_startup(|app| {
        let db = Db::init().expect("Failed to initialize database");
        let db_arc = Arc::new(db);
        *DB.lock().unwrap() = Some(db_arc.clone());

        // Load CSS
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("style.css"));
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // --- AppImage integration ---
        // Check if running as AppImage and integrate into application menu
        portals::integrate_appimage();

        // 2. Also request the XDG Background portal permission (Flatpak / portal-aware DEs).
        //    This runs async so it never blocks startup, and errors are just logged.
        let app_clone = app.clone();
        glib::MainContext::default().spawn_local(async move {
            // Setup keyboard shortcut
            let _ = portals::setup_shortcuts(&app_clone).await;

            // Try background portal (silently ignore errors for unsandboxed apps)
            if let Err(e) = portals::request_background().await {
                log::info!(
                    "Background portal unavailable (expected outside Flatpak): {}",
                    e
                );
            }
        });

        // Global Search Action
        let search_action = gio::SimpleAction::new("search", None);
        let app_clone_search = app.clone();
        search_action.connect_activate(move |_, _| {
            let search_win = adw::ApplicationWindow::builder()
                .application(&app_clone_search)
                .default_width(500)
                .default_height(60)
                .title("Spotlight Search")
                .decorated(false)
                .modal(true)
                .build();

            search_win.add_css_class("search-window");

            let overlay = gtk::Overlay::new();
            let entry = gtk::SearchEntry::builder()
                .placeholder_text("Search across all notes...")
                .hexpand(true)
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(12)
                .margin_end(12)
                .build();

            let list_box = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::None)
                .css_classes(["search-results"])
                .build();

            let popover = gtk::Popover::builder()
                .position(gtk::PositionType::Bottom)
                .child(&list_box)
                .autohide(false)
                .has_arrow(false)
                .build();

            popover.set_parent(&entry);

            let app_clone_for_entry = app_clone_search.clone();
            let search_win_clone = search_win.clone();
            entry.connect_search_changed(move |ent| {
                let query = ent.text().to_string();
                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }

                if query.len() > 1 {
                    if let Some(db) = DB.lock().unwrap().as_ref() {
                        if let Ok(blocks) = db.search_blocks(&query) {
                            if blocks.is_empty() {
                                popover.popdown();
                            } else {
                                for block in blocks.into_iter().take(5) {
                                    let btn = gtk::Button::with_label(&format!(
                                        "Note #{}: {}",
                                        block.note_id,
                                        block.content.chars().take(40).collect::<String>()
                                    ));
                                    let app_ref = app_clone_for_entry.clone();
                                    let win_ref = search_win_clone.clone();
                                    btn.connect_clicked(move |_| {
                                        win_ref.close();
                                        // Focus the matching note window
                                        for w in app_ref.windows() {
                                            if let Some(sticky) =
                                                w.downcast_ref::<crate::window::StickyWindow>()
                                            {
                                                if let Some(n) = sticky.imp().note.borrow().as_ref()
                                                {
                                                    if n.id == block.note_id {
                                                        sticky.present();
                                                        sticky.add_css_class("highlight");
                                                        glib::timeout_add_local_once(
                                                            std::time::Duration::from_millis(1500),
                                                            glib::clone!(
                                                                #[weak]
                                                                sticky,
                                                                move || {
                                                                    sticky.remove_css_class(
                                                                        "highlight",
                                                                    );
                                                                }
                                                            ),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    });
                                    list_box.append(&btn);
                                }
                                popover.popup();
                            }
                        }
                    }
                } else {
                    popover.popdown();
                }
            });

            overlay.set_child(Some(&entry));
            search_win.set_content(Some(&overlay));
            search_win.present();
            entry.grab_focus();
        });
        app.add_action(&search_action);
        app.set_accels_for_action("app.search", &["<Control><Shift>f"]);
        
        // Use std mpsc: tray thread sends a &'static str command,
        // a glib timeout polls and re-dispatches on the GTK thread.
        let (tray_tx, tray_rx) = mpsc::channel::<&'static str>();
        let tray_rx = Arc::new(Mutex::new(tray_rx));

        let app_for_tray = app.clone();
        let tray_rx_clone = tray_rx.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if let Ok(rx) = tray_rx_clone.lock() {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        "new-note" => portals::create_new_note(&app_for_tray),
                        "search" => app_for_tray.activate_action("search", None),
                        "restore" => {
                            if let Some(db) = DB.lock().unwrap().as_ref() {
                                if let Ok(Some(note)) = db.restore_last_deleted_note() {
                                    let window =
                                        crate::window::StickyWindow::new(&app_for_tray, note);
                                    window.present();
                                }
                            }
                        }
                        "empty" => {
                            if let Some(db) = DB.lock().unwrap().as_ref() {
                                let _ = db.empty_trash();
                            }
                        }
                        "export-backup" => {
                            let dialog = gtk::FileDialog::builder()
                                .title("Export Database Backup")
                                .initial_name("sticky_backup.db")
                                .build();
                            dialog.save(gtk::Window::NONE, gio::Cancellable::NONE, move |res| {
                                if let Ok(file) = res {
                                    if let Some(dest_path) = file.path() {
                                        let mut src_path = glib::user_data_dir();
                                        src_path.push("sticky");
                                        src_path.push("notes.db");
                                        let _ = std::fs::copy(&src_path, &dest_path);
                                    }
                                }
                            });
                        }
                        "import-backup" => {
                            let dialog = gtk::FileDialog::builder()
                                .title("Import Database Backup")
                                .build();
                            dialog.open(gtk::Window::NONE, gio::Cancellable::NONE, move |res| {
                                if let Ok(file) = res {
                                    if let Some(src_path) = file.path() {
                                        let mut dest_path = glib::user_data_dir();
                                        dest_path.push("sticky");
                                        dest_path.push("notes.db");
                                        let _ = std::fs::copy(&src_path, &dest_path);
                                        // Tell user to restart (we'll just log for now)
                                        log::info!("Backup imported. Restart Sticky to apply.");
                                    }
                                }
                            });
                        }
                        "quit" => app_for_tray.quit(),
                        _ => {}
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        std::thread::spawn(move || {
            struct AntitgravTray {
                tx: mpsc::Sender<&'static str>,
            }
            impl ksni::Tray for AntitgravTray {
                fn id(&self) -> String {
                    "sticky".into()
                }
                fn title(&self) -> String {
                    "Sticky Notes".into()
                }
                fn icon_name(&self) -> String {
                    "accessories-text-editor".into()
                }
                fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
                    use ksni::menu::*;
                    vec![
                        StandardItem {
                            label: "📝 New Note".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("new-note");
                            }),
                            ..Default::default()
                        }
                        .into(),
                        StandardItem {
                            label: "🔍 Search Notes".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("search");
                            }),
                            ..Default::default()
                        }
                        .into(),
                        MenuItem::Separator,
                        StandardItem {
                            label: "♻️ Restore Last Deleted".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("restore");
                            }),
                            ..Default::default()
                        }
                        .into(),
                        StandardItem {
                            label: "🗑️ Empty Trash".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("empty");
                            }),
                            ..Default::default()
                        }
                        .into(),
                        MenuItem::Separator,
                        StandardItem {
                            label: "💾 Export Backup".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("export-backup");
                            }),
                            ..Default::default()
                        }
                        .into(),
                        StandardItem {
                            label: "📂 Import Backup".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("import-backup");
                            }),
                            ..Default::default()
                        }
                        .into(),
                        MenuItem::Separator,
                        StandardItem {
                            label: "Quit".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("quit");
                            }),
                            ..Default::default()
                        }
                        .into(),
                    ]
                }
            }
            let service = ksni::TrayService::new(AntitgravTray { tx: tray_tx });
            service.spawn();
    });
    });

    app.connect_command_line(|app, cmdline| {
        println!("App command-line activated");
        let args: Vec<String> = cmdline.arguments().into_iter().map(|s| s.to_string_lossy().into_owned()).collect();
        let mut force_new = false;
        let mut background = false;
        let mut quit = false;
        
        for arg in args.iter().skip(1) {
            match arg.as_str() {
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
                    println!("Creating window for note ID: {}", note.id);
                    let window = window::StickyWindow::new(app, note.clone());
                    println!("Calling present for note ID: {}", note.id);
                    window.present();
                    println!("Present complete for note ID: {}", note.id);
                    active_windows.push(window);
                }
                
                // Keep the windows alive by storing them in a static RefCell if needed, 
                // but setting the application and presenting them should be enough in GTK4.
                // Just in case, we store them in a static to perfectly satisfy the requirement.
                WINDOWS.with(|w| {
                    *w.borrow_mut() = active_windows;
                });

                if app.windows().is_empty() {
                    println!("Fallback: No windows visible, creating a fresh one.");
                    let note_id = db_arc.create_note(300, 200, "#FFE66D").unwrap_or(1);
                    if let Ok(notes) = db_arc.get_notes() {
                        if let Some(note) = notes.into_iter().find(|n| n.id == note_id) {
                            let window = window::StickyWindow::new(app, note);
                            window.present();
                            WINDOWS.with(|w| {
                                w.borrow_mut().push(window);
                            });
                        }
                    }
                }
            } else {
                println!("Failed to get notes from DB");
            }
        } else {
            println!("DB not initialized");
        }

        // Force a new note creation if requested via CLI or if no windows visible and not running in background
        if force_new || (app.windows().is_empty() && !background) {
            portals::create_new_note(app);
        }

        0
    });
        app.run()
}
