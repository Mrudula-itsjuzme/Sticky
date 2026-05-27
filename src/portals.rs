use ashpd::desktop::Color;
use adw::prelude::*;
use gtk::gio;
use crate::DB;
use crate::window::StickyWindow;

pub async fn setup_shortcuts(app: &adw::Application) -> ashpd::Result<()> {
    let action = gio::SimpleAction::new("new-note", None);
    let app_clone = app.clone();
    action.connect_activate(move |_, _| {
        create_new_note(&app_clone);
    });
    app.add_action(&action);
    app.set_accels_for_action("app.new-note", &["<Control>n"]);
    
    Ok(())
}

pub async fn pick_color() -> ashpd::Result<Option<String>> {
    let color = Color::pick()
        .send()
        .await?
        .response()?;
    
    // Convert RGB to HEX
    let hex = format!("#{:02X}{:02X}{:02X}", 
        (color.red() * 255.0) as u8,
        (color.green() * 255.0) as u8,
        (color.blue() * 255.0) as u8
    );
    
    Ok(Some(hex))
}

pub fn create_new_note(app: &adw::Application) {
    if let Some(db) = DB.lock().unwrap().as_ref() {
        if let Ok(id) = db.create_note(100, 100, "#FFE66D") {
            if let Ok(notes) = db.get_notes() {
                if let Some(note) = notes.into_iter().find(|n| n.id == id) {
                    let window = StickyWindow::new(app, note);
                    window.present();
                }
            }
        }
    }
}
