use gtk::prelude::*;
use std::process::Command;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};
const APP_ICON: &[u8] = include_bytes!("../icon/trayIcon.ico");

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
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("warp-cli wrapper")
        .with_icon(load_tray_icon(APP_ICON))
        .build()
        .expect("Failed to build tray icon");

    // Spawn a thread to listen for menu events.
    // Each menu item is identified by its unique ID.
    // In the event loop thread:
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

    // Start the GTK main loop so that the tray icon remains active.
    gtk::main();
}
