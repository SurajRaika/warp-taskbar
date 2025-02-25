use gtk::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};

const APP_ICON: &[u8] = include_bytes!("../icon/trayIcon.ico");

#[derive(Deserialize)]
struct CommandItem {
    title: String,
    command: String,
}

fn load_tray_icon(image_data: &[u8]) -> Icon {
    let image = image::load_from_memory(image_data).expect("Failed to load icon image data");
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_flat_samples().samples;
    Icon::from_rgba(pixels, image.width(), image.height()).expect("Failed to create tray icon")
}

fn main() {
    // Initialize GTK (required for Linux/macOS tray applications)
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        return;
    }

    // Construct the path to the JSON file on your desktop.
    let home_dir = std::env::var("HOME").expect("Could not determine HOME directory");
    let commands_path = format!("{}/Desktop/commands.json", home_dir);

    // Read the JSON file.
    let file_content =
        fs::read_to_string(&commands_path).expect(&format!("Failed to read {}", commands_path));
    let commands: Vec<CommandItem> =
        serde_json::from_str(&file_content).expect("Failed to parse JSON from commands.json");

    // Create a mapping from unique menu IDs to their associated commands.
    let mut command_map: HashMap<String, String> = HashMap::new();

    // Create the tray menu.
    let tray_menu = Menu::new();

    // For each command from the JSON file, create a menu item.
    for (i, cmd_item) in commands.iter().enumerate() {
        // Generate a unique ID (e.g. "cmd0", "cmd1", ...)
        let menu_id = format!("cmd{}", i);
        // Save the command in our mapping.
        command_map.insert(menu_id.clone(), cmd_item.command.clone());
        // Create a menu item using the title from the JSON.
        let item = MenuItem::with_id(&menu_id, &cmd_item.title, true, None);
        tray_menu.append(&item).unwrap();
    }

    // Add a separator and a "Quit" option.
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    tray_menu.append(&quit_item).unwrap();

    // Build the tray icon with the dynamically built menu.
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Dynamic Commands Tray")
        .with_icon(load_tray_icon(APP_ICON))
        .build()
        .expect("Failed to build tray icon");

    // Share the command map between threads.
    let command_map = Arc::new(command_map);

    // Spawn a thread to listen for menu events.
    let cmd_map = command_map.clone();
    std::thread::spawn(move || {
        loop {
            match MenuEvent::receiver().recv() {
                Ok(event) => {
                    // Access the inner string of MenuId using .0.
                    let id = event.id.0;
                    if id == "quit" {
                        println!("Quitting application...");
                        gtk::main_quit();
                        break;
                    } else if id.starts_with("cmd") {
                        if let Some(cmd) = cmd_map.get(&id) {
                            println!("Executing command: {}", cmd);
                            // Use:
                            match Command::new("sh").arg("-c").arg(cmd).output() {
                                Ok(output) => {
                                    println!(
                                        "Output:\n{}",
                                        String::from_utf8_lossy(&output.stdout)
                                    );
                                }
                                Err(e) => eprintln!("Error executing {}: {}", cmd, e),
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error receiving menu event: {}", e),
            }
        }
    });

    // Run the GTK main loop so that the tray icon remains active.
    gtk::main();
}
