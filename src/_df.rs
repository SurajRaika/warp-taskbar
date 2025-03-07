use gtk::prelude::*;
use std::{process::Command, time::Duration};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, MenuItemKind, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};
use std::env;
use std::fs;
use std::path::Path;

static TRAY_ICON_DARK_ACTIVE: &[u8] = include_bytes!("../icon/cloudflare-dark-active.ico");
static TRAY_ICON_INACTIVE: &[u8] = include_bytes!("../icon/cloudflare-inactive.ico");
static TRAY_ICON_LIGHT_ACTIVE: &[u8] = include_bytes!("../icon/cloudflare-light-active.ico");

pub fn is_dark_mode_enabled() -> bool {
    // Check for GNOME
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output() 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("dark") {
            return true;
        }
    }
    
    // Check for KDE Plasma
    if let Some(home) = env::var_os("HOME") {
        let kde_config_path = Path::new(&home).join(".config").join("kdeglobals");
        if kde_config_path.exists() {
            if let Ok(content) = fs::read_to_string(kde_config_path) {
                // KDE has multiple ways to specify dark theme
                if content.contains("[Colors:View]") && content.contains("BackgroundNormal=") {
                    // Check for dark background color values
                    // This is a simplification; a more robust implementation would parse the color values
                    if content.contains("BackgroundNormal=35,38,41") {
                        return true;
                    }
                }
                
                // Check if a dark theme is explicitly set
                if content.contains("ColorScheme=BreezeDark") || 
                   content.contains("name=Breeze Dark") {
                    return true;
                }
            }
        }
    }
    
    // Check for XFCE
    if let Ok(output) = Command::new("xfconf-query")
        .args(["-c", "xsettings", "-p", "/Net/ThemeName"])
        .output() 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("dark") || stdout.contains("Dark") {
            return true;
        }
    }
    
    // Check for Cinnamon
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.cinnamon.desktop.interface", "gtk-theme"])
        .output() 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("dark") || stdout.contains("Dark") {
            return true;
        }
    }
    
    // Check for MATE
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.mate.interface", "gtk-theme"])
        .output() 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("dark") || stdout.contains("Dark") {
            return true;
        }
    }
    
    // Check for Elementary OS
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "io.elementary.terminal.settings", "prefer-dark-style"])
        .output() 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("true") {
            return true;
        }
    }
    
    // Fallback to checking GTK theme settings in general
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output() 
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("dark") || stdout.contains("Dark") {
            return true;
        }
    }
    
    false
}

fn get_active_tray_icon() -> &'static [u8] {
    if is_dark_mode_enabled() {
        TRAY_ICON_LIGHT_ACTIVE
    } else {
        TRAY_ICON_DARK_ACTIVE
    }
}

fn is_warp_disconnected() -> bool {
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

// Helper function to run warp-cli commands
fn run_warp_command(command: &str, args: &[&str]) {
    println!("Executing: warp-cli {} {}", command, args.join(" "));
    match Command::new("warp-cli")
        .arg(command)
        .args(args)
        .output()
    {
        Ok(output) => {
            println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
        }
        Err(e) => eprintln!("Error running {}: {}", command, e),
    }
}

fn main() {
    // Initialize GTK (required for Linux/macOS; on Windows you'd run the win32 event loop)
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        return;
    }

    // Create the main tray menu
    let tray_menu = Menu::new();
    
    // Basic connectivity options
    let connect_item = MenuItem::with_id("connect", "Connect", true, None);
    let disconnect_item = MenuItem::with_id("disconnect", "Disconnect", true, None);
    let status_item = MenuItem::with_id("status", "Status", true, None);
    
    // Create the "On Startup" submenu
    let startup_menu = Menu::new();
    let enable_always_on = MenuItem::with_id("enable_always_on", "Enable Always-On", true, None);
    let disable_always_on = MenuItem::with_id("disable_always_on", "Disable Always-On", true, None);
    startup_menu.append(&enable_always_on).unwrap();
    startup_menu.append(&disable_always_on).unwrap();
    let startup_item = MenuItem::with_id_and_submenu("startup", "On Startup", true, startup_menu);
    
    // Create the "Set Mode" submenu
    let mode_menu = Menu::new();
    let mode_warp = MenuItem::with_id("mode_warp", "WARP", true, None);
    let mode_doh = MenuItem::with_id("mode_doh", "DoH (DNS over HTTPS)", true, None);
    let mode_dot = MenuItem::with_id("mode_dot", "DoT (DNS over TLS)", true, None);
    let mode_warp_doh = MenuItem::with_id("mode_warp_doh", "WARP+DoH", true, None);
    let mode_warp_dot = MenuItem::with_id("mode_warp_dot", "WARP+DoT", true, None);
    mode_menu.append(&mode_warp).unwrap();
    mode_menu.append(&mode_doh).unwrap();
    mode_menu.append(&mode_dot).unwrap();
    mode_menu.append(&mode_warp_doh).unwrap();
    mode_menu.append(&mode_warp_dot).unwrap();
    let mode_item = MenuItem::with_id_and_submenu("mode", "Set Mode", true, mode_menu);
    
    // Create the "Other" submenu
    let other_menu = Menu::new();
    let teams_unenroll = MenuItem::with_id("teams_unenroll", "Unenroll from Cloudflare for Teams", true, None);
    let register = MenuItem::with_id("register", "Register Device with Cloudflare", true, None);
    let enable_logging = MenuItem::with_id("enable_logging", "Enable Debug Logging", true, None);
    let disable_logging = MenuItem::with_id("disable_logging", "Disable Debug Logging", true, None);
    let trace_support = MenuItem::with_id("trace_support", "Generate Trace Report", true, None);
    let generate_report = MenuItem::with_id("generate_report", "Generate Diagnostic Report", true, None);
    other_menu.append(&teams_unenroll).unwrap();
    other_menu.append(&register).unwrap();
    
    // Add a separator in the Other menu
    let separator_item = MenuItem::with_kind(MenuItemKind::Separator);
    other_menu.append(&separator_item).unwrap();
    
    other_menu.append(&enable_logging).unwrap();
    other_menu.append(&disable_logging).unwrap();
    
    // Add another separator
    let separator_item2 = MenuItem::with_kind(MenuItemKind::Separator);
    other_menu.append(&separator_item2).unwrap();
    
    other_menu.append(&trace_support).unwrap();
    other_menu.append(&generate_report).unwrap();
    let other_item = MenuItem::with_id_and_submenu("other", "Other", true, other_menu);
    
    // Quit item
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    
    // Add a separator before the Save option
    let separator_item3 = MenuItem::with_kind(MenuItemKind::Separator);
    tray_menu.append(&separator_item3).unwrap();
    
    // Save option (as mentioned in your requirements)
    let save_item = MenuItem::with_id("save", "Save", true, None);
    
    // Append all items to the main menu
    tray_menu.append(&connect_item).unwrap();
    tray_menu.append(&disconnect_item).unwrap();
    tray_menu.append(&status_item).unwrap();
    
    // Add a separator
    let separator_item4 = MenuItem::with_kind(MenuItemKind::Separator);
    tray_menu.append(&separator_item4).unwrap();
    
    tray_menu.append(&startup_item).unwrap();
    tray_menu.append(&mode_item).unwrap();
    tray_menu.append(&other_item).unwrap();
    
    // Add a separator before the Save option
    let separator_item5 = MenuItem::with_kind(MenuItemKind::Separator);
    tray_menu.append(&separator_item5).unwrap();
    
    tray_menu.append(&save_item).unwrap();
    
    // Add another separator before Quit
    let separator_item6 = MenuItem::with_kind(MenuItemKind::Separator);
    tray_menu.append(&separator_item6).unwrap();
    
    tray_menu.append(&quit_item).unwrap();

    // Build the tray icon with the above menu and icon image.
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Cloudflare WARP Controller")
        .with_icon(load_tray_icon(APP_ICONS.cloudflare_inactive))
        .build()
        .expect("Failed to build tray icon");

    // Clone the tray icon so it can be used inside the timeout closure.
    let tray_icon_ptr = tray_icon.clone();

    // Spawn a thread to listen for menu events.
    std::thread::spawn(|| {
        loop {
            match MenuEvent::receiver().recv() {
                Ok(event) => match event.id.0.as_str() {
                    // Basic operations
                    "connect" => run_warp_command("connect", &[]),
                    "disconnect" => run_warp_command("disconnect", &[]),
                    "status" => run_warp_command("status", &[]),
                    "save" => println!("Save functionality not implemented"),
                    
                    // Startup options
                    "enable_always_on" => run_warp_command("enable-always-on", &[]),
                    "disable_always_on" => run_warp_command("disable-always-on", &[]),
                    
                    // Mode options
                    "mode_warp" => run_warp_command("set-mode", &["warp"]),
                    "mode_doh" => run_warp_command("set-mode", &["doh"]),
                    "mode_dot" => run_warp_command("set-mode", &["dot"]),
                    "mode_warp_doh" => run_warp_command("set-mode", &["warp+doh"]),
                    "mode_warp_dot" => run_warp_command("set-mode", &["warp+dot"]),
                    
                    // Other options
                    "teams_unenroll" => run_warp_command("teams-unenroll", &[]),
                    "register" => run_warp_command("register", &[]),
                    "enable_logging" => run_warp_command("enable-logging", &[]),
                    "disable_logging" => run_warp_command("disable-logging", &[]),
                    "trace_support" => run_warp_command("trace-support", &[]),
                    "generate_report" => run_warp_command("generate-report", &[]),
                    
                    // Quit
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

    // Set up a GLib timeout to check the connection status every 2 seconds.
    glib::timeout_add_local(Duration::from_secs(2), move || {
        let is_inactive = is_warp_disconnected();

        if is_inactive {
            tray_icon_ptr.set_icon(Some(load_tray_icon(TRAY_ICON_INACTIVE)));
        } else {
            tray_icon_ptr.set_icon(Some(load_tray_icon(get_active_tray_icon())));
        }
        // Continue the timeout indefinitely.
        glib::ControlFlow::Continue
    });

    // Start the GTK main loop so the tray icon remains active.
    gtk::main();
}