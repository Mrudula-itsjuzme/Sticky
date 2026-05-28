use ashpd::desktop::Color;
use ashpd::desktop::background::Background;
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

/// Request OS-level permission to run in the background and autostart.
/// **Note**: The XDG Background portal only works inside Flatpak sandboxes.
/// For unsandboxed installs we fall back to writing a ~/.config/autostart/ entry.
pub async fn request_background() -> ashpd::Result<()> {
    let response = Background::request()
        .reason("Keep your sticky notes alive across logins and reboots")
        .auto_start(true)
        .dbus_activatable(false)
        .send()
        .await?
        .response()?;

    log::info!(
        "Background portal: run_in_background={}, auto_start={}",
        response.run_in_background(),
        response.auto_start()
    );
    Ok(())
}

/// Write an XDG autostart .desktop file so the app relaunches on every login.
/// This works on any desktop environment (GNOME, KDE, Xfce, …).
pub fn install_autostart() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Could not determine own executable path: {}", e);
            return;
        }
    };

    let mut autostart_dir = glib::user_config_dir();
    autostart_dir.push("autostart");

    if let Err(e) = std::fs::create_dir_all(&autostart_dir) {
        log::warn!("Could not create autostart dir: {}", e);
        return;
    }

    autostart_dir.push("sticky.desktop");

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Sticky Notes\n\
         Comment=Sticky notes that survive shutdowns and reboots\n\
         Exec={}\n\
         Icon=accessories-text-editor\n\
         Hidden=false\n\
         NoDisplay=false\n\
         X-GNOME-Autostart-enabled=true\n",
        exe.display()
    );

    match std::fs::write(&autostart_dir, &content) {
        Ok(_) => log::info!("Autostart entry written to {:?}", autostart_dir),
        Err(e) => log::warn!("Could not write autostart entry: {}", e),
    }
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
