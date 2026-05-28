use crate::window::StickyWindow;
use crate::DB;
use adw::prelude::*;
use ashpd::desktop::background::Background;
use ashpd::desktop::Color;
use gtk::gio;

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
/// This file goes into ~/.config/autostart/ and is **not** a launcher entry.
/// NoDisplay=true ensures it never appears in the application menu.
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

    autostart_dir.push("sticky-autostart.desktop");

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Sticky Autostart\n\
         Comment=Auto-start Sticky on login\n\
         Exec={} --background\n\
         Icon=sticky\n\
         Hidden=false\n\
         NoDisplay=true\n\
         X-GNOME-Autostart-enabled=true\n",
        exe.display()
    );

    match std::fs::write(&autostart_dir, &content) {
        Ok(_) => log::info!("Autostart entry written to {:?}", autostart_dir),
        Err(e) => log::warn!("Could not write autostart entry: {}", e),
    }
}

/// If running inside an AppImage, automatically register the app in the user's Application Menu.
/// This only creates a launcher when the APPIMAGE env var is present (set by AppImage runtime).
pub fn integrate_appimage() {
    if let Ok(appimage_path) = std::env::var("APPIMAGE") {
        let mut apps_dir = glib::user_data_dir();
        apps_dir.push("applications");

        if std::fs::create_dir_all(&apps_dir).is_ok() {
            let desktop_path = apps_dir.join("sticky-appimage.desktop");

            let content = format!(
                "[Desktop Entry]\n\
                 Type=Application\n\
                 Name=Sticky\n\
                 Comment=Modern floating sticky notes and infinite whiteboard\n\
                 Exec=\"{}\"\n\
                 Icon=sticky\n\
                 Terminal=false\n\
                 Categories=Utility;GTK;\n\
                 StartupNotify=true\n",
                appimage_path
            );

            if std::fs::write(&desktop_path, &content).is_ok() {
                log::info!(
                    "AppImage integrated into Application Menu: {:?}",
                    desktop_path
                );

                // Refresh the desktop database so the launcher picks it up immediately
                let _ = std::process::Command::new("update-desktop-database")
                    .arg(&apps_dir)
                    .spawn()
                    .and_then(|mut c| c.wait());
            }
        }
    }
}

pub async fn pick_color() -> ashpd::Result<Option<String>> {
    let color = Color::pick().send().await?.response()?;

    // Convert RGB to HEX
    let hex = format!(
        "#{:02X}{:02X}{:02X}",
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
