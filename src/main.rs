mod db;
mod window;
mod canvas;
mod text_block;
mod portals;

use adw::prelude::*;
use adw::subclass::prelude::ObjectSubclassIsExt;
use adw::Application;
use db::Db;
use gtk::{gdk, gio};
use std::sync::{Arc, Mutex, mpsc};
use once_cell::sync::Lazy;

pub static DB: Lazy<Mutex<Option<Arc<Db>>>> = Lazy::new(|| Mutex::new(None));
pub static TOKIO_RT: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
});

fn main() -> glib::ExitCode {
    env_logger::init();

    let app = adw::Application::builder()
        .application_id("com.mrudula.sticky")
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

        // --- Autostart / background persistence ---
        // 1. Write ~/.config/autostart/antigrav.desktop immediately (works everywhere).
        portals::install_autostart();

        // 2. Also request the XDG Background portal permission (Flatpak / portal-aware DEs).
        //    This runs async so it never blocks startup, and errors are just logged.
        let app_clone = app.clone();
        glib::MainContext::default().spawn_local(async move {
            // Setup keyboard shortcut
            let _ = portals::setup_shortcuts(&app_clone).await;

            // Try background portal (silently ignore errors for unsandboxed apps)
            if let Err(e) = portals::request_background().await {
                log::info!("Background portal unavailable (expected outside Flatpak): {}", e);
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
                                    let btn = gtk::Button::with_label(&format!("Note #{}: {}", block.note_id, block.content.chars().take(40).collect::<String>()));
                                    let app_ref = app_clone_for_entry.clone();
                                    let win_ref = search_win_clone.clone();
                                    btn.connect_clicked(move |_| {
                                        win_ref.close();
                                        // Focus the matching note window
                                        for w in app_ref.windows() {
                                            if let Some(sticky) = w.downcast_ref::<crate::window::StickyWindow>() {
                                                if let Some(n) = sticky.imp().note.borrow().as_ref() {
                                                    if n.id == block.note_id {
                                                        sticky.present();
                                                        sticky.add_css_class("highlight");
                                                        glib::timeout_add_local_once(
                                                            std::time::Duration::from_millis(1500),
                                                            glib::clone!(#[weak] sticky, move || {
                                                                sticky.remove_css_class("highlight");
                                                            })
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

    });

    app.connect_activate(|app| {
        println!("App activated");
        if let Some(db_arc) = DB.lock().unwrap().as_ref().cloned() {
            if let Ok(notes) = db_arc.get_notes() {
                println!("Found {} notes in DB", notes.len());
                if notes.is_empty() {
                    println!("No notes found, creating a default one...");
                    let _ = db_arc.create_note(100, 100, "#FFE66D");
                }
                
                for note in db_arc.get_notes().unwrap_or_default() {
                    println!("Creating window for note ID: {}", note.id);
                    let window = window::StickyWindow::new(app, note);
                    window.present();
                }
            } else {
                println!("Failed to get notes from DB");
            }
        } else {
            println!("DB not initialized");
        }

        // --- System Tray Icon ---
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
                        "search"   => app_for_tray.activate_action("search", None),
                        "restore"  => {
                            if let Some(db) = DB.lock().unwrap().as_ref() {
                                if let Ok(Some(note)) = db.restore_last_deleted_note() {
                                    let window = crate::window::StickyWindow::new(&app_for_tray, note);
                                    window.present();
                                }
                            }
                        },
                        "empty"    => {
                            if let Some(db) = DB.lock().unwrap().as_ref() {
                                let _ = db.empty_trash();
                            }
                        },
                        "quit"     => app_for_tray.quit(),
                        _ => {}
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        std::thread::spawn(move || {
            struct AntitgravTray { tx: mpsc::Sender<&'static str> }
            impl ksni::Tray for AntitgravTray {
                fn id(&self) -> String { "sticky".into() }
                fn title(&self) -> String { "Sticky Notes".into() }
                fn icon_name(&self) -> String { "accessories-text-editor".into() }
                fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
                    use ksni::menu::*;
                    vec![
                        StandardItem {
                            label: "📝 New Note".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("new-note");
                            }),
                            ..Default::default()
                        }.into(),
                        StandardItem {
                            label: "🔍 Search Notes".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("search");
                            }),
                            ..Default::default()
                        }.into(),
                        MenuItem::Separator,
                        StandardItem {
                            label: "♻️ Restore Last Deleted".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("restore");
                            }),
                            ..Default::default()
                        }.into(),
                        StandardItem {
                            label: "🗑️ Empty Trash".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("empty");
                            }),
                            ..Default::default()
                        }.into(),
                        MenuItem::Separator,
                        StandardItem {
                            label: "Quit".into(),
                            activate: Box::new(|this: &mut AntitgravTray| {
                                let _ = this.tx.send("quit");
                            }),
                            ..Default::default()
                        }.into(),
                    ]
                }
            }
            let service = ksni::TrayService::new(AntitgravTray { tx: tray_tx });
            service.spawn();
            loop { std::thread::sleep(std::time::Duration::from_secs(3600)); }
        });
    });

    app.run()
}
