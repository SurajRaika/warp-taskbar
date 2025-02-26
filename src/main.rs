use gtk::prelude::*;
use std::{process::Command, time::Duration};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

fn condition() -> bool {
    use std::process::Command;
    let output = Command::new("warp-cli")
        .arg("status")
        .output()
        .expect("Failed to execute warp-cli status command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("Status update: Disconnected")
}

struct AppIcons {
    cloudflare_dark_active: &'static [u8],
    cloudflare_inactive: &'static [u8],
    cloudflare_light_active: &'static [u8],
}

const APP_ICONS: AppIcons = AppIcons {
    cloudflare_dark_active: include_bytes!("../icon/cloudflare-dark-active.ico"),
    cloudflare_inactive: include_bytes!("../icon/cloudflare-inactive.ico"),
    cloudflare_light_active: include_bytes!("../icon/cloudflare-light-active.ico"),
};

fn load_tray_icon(image_data: &[u8]) -> Icon {
    let image = image::load_from_memory(image_data).expect("Failed to load icon image data");
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_flat_samples().samples;
    Icon::from_rgba(pixels, image.width(), image.height()).expect("Failed to create tray icon")
}

fn main() {
    // Initialize GTK (required for Linux/macOS; on Windows you’d run the win32 event loop)
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        return;
    }

    // Create the tray menu with warp-cli related options.
    let tray_menu = Menu::new();
    let connect_item = MenuItem::with_id("connect", "Warp Connect", true, None);
    let disconnect_item = MenuItem::with_id("disconnect", "Warp Disconnect", true, None);
    let status_item = MenuItem::with_id("status", "Warp Status", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);

    // Append the menu items (order is as desired)
    tray_menu.append(&connect_item).unwrap();
    tray_menu.append(&disconnect_item).unwrap();
    tray_menu.append(&status_item).unwrap();
    tray_menu.append(&quit_item).unwrap();

    // Build the tray icon with the above menu and icon image.
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("warp-cli wrapper")
        .with_icon(load_tray_icon(APP_ICONS.cloudflare_inactive))
        .build()
        .expect("Failed to build tray icon");

    // Clone the tray icon so it can be used inside the timeout closure.
    let tray_icon_ptr = tray_icon.clone();

    // A simple toggle variable to simulate condition changes.
    let mut toggle = false;
    // Spawn a thread to listen for menu events.
    // Each menu item is identified by its unique ID.
    // In the event loop thread:
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    std::thread::spawn(|| {
        loop {
            match tray_icon::menu::MenuEvent::receiver().recv() {
                Ok(event) => match event.id.0.as_str() {
                    // <-- use .0 to access the inner String
                    "connect" => {
                        println!("Executing: warp-cli connect");
                        match std::process::Command::new("warp-cli")
                            .arg("connect")
                            .output()
                        {
                            Ok(output) => {
                                println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                            }
                            Err(e) => eprintln!("Error running connect: {}", e),
                        }
                    }
                    "disconnect" => {
                        println!("Executing: warp-cli disconnect");
                        match std::process::Command::new("warp-cli")
                            .arg("disconnect")
                            .output()
                        {
                            Ok(output) => {
                                println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                            }
                            Err(e) => eprintln!("Error running disconnect: {}", e),
                        }
                    }
                    "status" => {
                        println!("Executing: warp-cli status");
                        match std::process::Command::new("warp-cli")
                            .arg("status")
                            .output()
                        {
                            Ok(output) => {
                                println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                            }
                            Err(e) => eprintln!("Error running status: {}", e),
                        }
                    }
                    "quit" => {
                        println!("Quitting application...");
                        gtk::main_quit();
                        break;
                    }
                    _ => {}
                },
                Err(e) => eprintln!("Error receiving menu event: {}", e),
            }
        }
    });

    // Set up a GLib timeout to check the condition every 2 seconds.
    glib::timeout_add_local(Duration::from_secs(2), move || {
        // Toggle the condition value.
        // toggle = !toggle;
        // Alternatively, you could call: let is_active = condition();
        let is_active = condition();

        if is_active {
            tray_icon_ptr.set_icon(Some(load_tray_icon(APP_ICONS.cloudflare_dark_active)));
        } else {
            tray_icon_ptr.set_icon(Some(load_tray_icon(APP_ICONS.cloudflare_inactive)));
        }
        // Continue the timeout indefinitely.
        glib::ControlFlow::Continue
    });

    // Start the GTK main loop so the tray icon remains active.
    gtk::main();
}
