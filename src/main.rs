use gtk;
use std::env;
use std::fs;
use std::path::Path;
use std::{process::Command, time::Duration};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

// Include your icons
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
                if content.contains("[Colors:View]") && content.contains("BackgroundNormal=") {
                    if content.contains("BackgroundNormal=35,38,41") {
                        return true;
                    }
                }
                if content.contains("ColorScheme=BreezeDark")
                    || content.contains("name=Breeze Dark")
                {
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
        .args([
            "get",
            "io.elementary.terminal.settings",
            "prefer-dark-style",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("true") {
            return true;
        }
    }

    // Fallback: check GTK theme setting in general
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
    // Initialize GTK (needed on Linux/macOS)
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        return;
    }

    // Create a flat tray menu (no nested submenus)
    let tray_menu = Menu::new();
    let connect_item = MenuItem::with_id("connect", "Warp Connect", true, None);
    let disconnect_item = MenuItem::with_id("disconnect", "Warp Disconnect", true, None);
    let status_item = MenuItem::with_id("status", "Warp Status", true, None);

    // Instead of a submenu for startup options, we prefix the labels
    let enable_always_on_item = MenuItem::with_id(
        "enable_always_on",
        "On StartUp: warp-cli enable-always-on",
        true,
        None,
    );
    let disable_always_on_item = MenuItem::with_id(
        "disable_always_on",
        "On StartUp: warp-cli disable-always-on",
        true,
        None,
    );

    // Flatten set mode options
    let set_mode_warp_item = MenuItem::with_id("set_mode_warp", "Set Mode: warp", true, None);
    let set_mode_doh_item = MenuItem::with_id("set_mode_doh", "Set Mode: doh", true, None);
    let set_mode_dot_item = MenuItem::with_id("set_mode_dot", "Set Mode: dot", true, None);
    let set_mode_warp_doh_item =
        MenuItem::with_id("set_mode_warp_doh", "Set Mode: warp+doh", true, None);
    let set_mode_warp_dot_item =
        MenuItem::with_id("set_mode_warp_dot", "Set Mode: warp+dot", true, None);

    // Flatten "Other" options
    let teams_unenroll_item = MenuItem::with_id(
        "teams_unenroll",
        "Other: warp-cli teams-unenroll",
        true,
        None,
    );
    let register_item = MenuItem::with_id("register", "Other: warp-cli register", true, None);
    let enable_logging_item = MenuItem::with_id(
        "enable_logging",
        "Other: warp-cli enable-logging",
        true,
        None,
    );
    let disable_logging_item = MenuItem::with_id(
        "disable_logging",
        "Other: warp-cli disable-logging",
        true,
        None,
    );
    let trace_support_item =
        MenuItem::with_id("trace_support", "Other: warp-cli trace-support", true, None);
    let generate_report_item = MenuItem::with_id(
        "generate_report",
        "Other: warp-cli generate-report",
        true,
        None,
    );

    // Append all items to the tray menu
    tray_menu.append(&connect_item).unwrap();
    tray_menu.append(&disconnect_item).unwrap();
    tray_menu.append(&status_item).unwrap();
    tray_menu.append(&enable_always_on_item).unwrap();
    tray_menu.append(&disable_always_on_item).unwrap();
    tray_menu.append(&set_mode_warp_item).unwrap();
    tray_menu.append(&set_mode_doh_item).unwrap();
    tray_menu.append(&set_mode_dot_item).unwrap();
    tray_menu.append(&set_mode_warp_doh_item).unwrap();
    tray_menu.append(&set_mode_warp_dot_item).unwrap();
    tray_menu.append(&teams_unenroll_item).unwrap();
    tray_menu.append(&register_item).unwrap();
    tray_menu.append(&enable_logging_item).unwrap();
    tray_menu.append(&disable_logging_item).unwrap();
    tray_menu.append(&trace_support_item).unwrap();
    tray_menu.append(&generate_report_item).unwrap();

    // Build the tray icon with the menu and initial icon.
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("warp-cli wrapper")
        .with_icon(load_tray_icon(APP_ICONS.cloudflare_inactive))
        .build()
        .expect("Failed to build tray icon");

    // Clone the tray icon for use in our periodic update thread.
    let tray_icon_ptr = tray_icon.clone();

    // Spawn a thread to listen for menu events.
    std::thread::spawn(|| loop {
        match MenuEvent::receiver().recv() {
            Ok(event) => match event.id.0.as_str() {
                "connect" => {
                    println!("Executing: warp-cli connect");
                    if let Ok(output) = Command::new("warp-cli").arg("connect").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "disconnect" => {
                    println!("Executing: warp-cli disconnect");
                    if let Ok(output) = Command::new("warp-cli").arg("disconnect").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "status" => {
                    println!("Executing: warp-cli status");
                    if let Ok(output) = Command::new("warp-cli").arg("status").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "enable_always_on" => {
                    println!("Executing: warp-cli enable-always-on");
                    if let Ok(output) = Command::new("warp-cli").arg("enable-always-on").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "disable_always_on" => {
                    println!("Executing: warp-cli disable-always-on");
                    if let Ok(output) = Command::new("warp-cli").arg("disable-always-on").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "set_mode_warp" => {
                    println!("Executing: warp-cli set-mode warp");
                    if let Ok(output) = Command::new("warp-cli").args(["set-mode", "warp"]).output()
                    {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "set_mode_doh" => {
                    println!("Executing: warp-cli set-mode doh");
                    if let Ok(output) = Command::new("warp-cli").args(["set-mode", "doh"]).output()
                    {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "set_mode_dot" => {
                    println!("Executing: warp-cli set-mode dot");
                    if let Ok(output) = Command::new("warp-cli").args(["set-mode", "dot"]).output()
                    {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "set_mode_warp_doh" => {
                    println!("Executing: warp-cli set-mode warp+doh");
                    if let Ok(output) = Command::new("warp-cli")
                        .args(["set-mode", "warp+doh"])
                        .output()
                    {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "set_mode_warp_dot" => {
                    println!("Executing: warp-cli set-mode warp+dot");
                    if let Ok(output) = Command::new("warp-cli")
                        .args(["set-mode", "warp+dot"])
                        .output()
                    {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "teams_unenroll" => {
                    println!("Executing: warp-cli teams-unenroll");
                    if let Ok(output) = Command::new("warp-cli").arg("teams-unenroll").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "register" => {
                    println!("Executing: warp-cli register");
                    if let Ok(output) = Command::new("warp-cli").arg("register").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "enable_logging" => {
                    println!("Executing: warp-cli enable-logging");
                    if let Ok(output) = Command::new("warp-cli").arg("enable-logging").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "disable_logging" => {
                    println!("Executing: warp-cli disable-logging");
                    if let Ok(output) = Command::new("warp-cli").arg("disable-logging").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "trace_support" => {
                    println!("Executing: warp-cli trace-support");
                    if let Ok(output) = Command::new("warp-cli").arg("trace-support").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                "generate_report" => {
                    println!("Executing: warp-cli generate-report");
                    if let Ok(output) = Command::new("warp-cli").arg("generate-report").output() {
                        println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                    }
                }
                _ => {}
            },
            Err(e) => eprintln!("Error receiving menu event: {}", e),
        }
    });

    // Set up a GLib timeout to update the tray icon every 2 seconds.
    glib::timeout_add_local(Duration::from_secs(2), move || {
        if is_warp_disconnected() {
            tray_icon_ptr.set_icon(Some(load_tray_icon(TRAY_ICON_INACTIVE)));
        } else {
            tray_icon_ptr.set_icon(Some(load_tray_icon(get_active_tray_icon())));
        }
        glib::ControlFlow::Continue
    });

    // Start the GTK main loop.
    gtk::main();
}
