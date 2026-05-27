mod db;
mod window;
mod canvas;
mod text_block;
mod portals;

use adw::prelude::*;
use adw::Application;
use db::Db;
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static DB: Lazy<Mutex<Option<Arc<Db>>>> = Lazy::new(|| Mutex::new(None));

fn main() -> glib::ExitCode {
    env_logger::init();

    let app = Application::builder()
        .application_id("com.antigravity.antigrav")
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

        // Setup shortcuts
        let app_clone = app.clone();
        glib::MainContext::default().spawn_local(async move {
            let _ = portals::setup_shortcuts(&app_clone).await;
        });
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
    });

    app.run()
}
