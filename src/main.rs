#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::path::Path;
use std::rc::Rc;

slint::include_modules!();

mod audio;

use serde::Deserialize;
use slint::{Image, ModelRc, VecModel};

#[derive(Deserialize, Clone)]
struct ItemConfig {
    label: String,
    program_path: Option<String>,
    runnable: bool,
}

fn app_dir() -> std::path::PathBuf {
    if cfg!(debug_assertions) { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")) }
    else { std::env::current_exe().unwrap().parent().unwrap().to_path_buf() }
}

fn load_config() -> Vec<ItemConfig> {
    let path = app_dir().join("config.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn model<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(v)))
}

fn to_launcher_item(c: &ItemConfig, icons: &Path) -> LauncherItem {
    let icon_path = icons.join(format!("{}.png", c.label));
    LauncherItem {
        label: c.label.to_uppercase().into(),
        icon: if icon_path.exists() {
            Image::load_from_path(&icon_path).unwrap_or_default()
        } else {
            Image::default()                 // blank image, no load attempted
        },
    }
}

// creates 'window' or sub-array from array
fn window(start: i32, items: &[ItemConfig], len: usize, icons: &Path) -> Vec<LauncherItem> {
    (0..len as i32).map(|c| {
        let idx = (start + c).rem_euclid(items.len() as i32) as usize;
        to_launcher_item(&items[idx], icons)
    }).collect()
}


// creates both 'windowed' arrays
fn build_windows(ui: &AppWindow, config: &[ItemConfig], icons: &Path, sel: i32) {
    let prev: Vec<_> = window(config.len() as i32 - 4 + sel, config, 5, icons)
        .into_iter().rev().collect();
    
    let next = window(sel, config, 7, icons);
    
    let app = ui.global::<AppState>();
    app.set_items_prev(model(prev));
    app.set_items_next(model(next));
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let app = ui.global::<AppState>();

    let config = load_config();
    let icons = app_dir().join("icons");

    let sfx = std::rc::Rc::new(audio::Audio::new());

    if config.is_empty() {
        eprintln!("config.json missing or empty — nothing to show");
        ui.run()?;
        return Ok(());
    }

    // initial card fill, before any key is pressed
    build_windows(&ui, &config, &icons, app.get_selected());

    app.on_play_select({
        let sfx = sfx.clone();
        move || sfx.select()
    });

    app.on_play_open({
        let sfx = sfx.clone();
        move || sfx.open()
    });

    app.on_play_equip({
        let sfx = sfx.clone();
        move || sfx.equip()
    });

    app.on_play_end({
        let sfx = sfx.clone();
        move || sfx.end()
    });

    app.on_play_bad({
        let sfx = sfx.clone();
        move || sfx.bad()
    });

    // shift the window each time a step settles
    app.on_settle({
        let ui = ui.as_weak();
        let config = config.clone();
        let icons = icons.clone();
        move || {
            let ui = ui.unwrap();
            let app = ui.global::<AppState>();
            
            let sel = (app.get_selected() + app.get_dir()).rem_euclid(config.len() as i32);
            // println!("index {:?} selected", sel);
            app.set_selected(sel);
            
            build_windows(&ui, &config, &icons, sel);
        }
    });

    app.on_run({
        let ui = ui.as_weak();
        let config = config.clone();
        let sfx = sfx.clone();

        move || {
            let sel = ui.unwrap().global::<AppState>().get_selected();
            
            // if there is an item in config at the index/sel that fits ItemConfig schema
            if let Some(item) = config.get(sel as usize) {
                // if runnable bool on item is true
                if item.runnable {
                    // if the item has a program path that is a string
                    if let Some(path) = &item.program_path {
                        let looks_like_path =
                            path.contains(':') || path.contains('\\') || path.contains('/');

                        let _ = if looks_like_path {
                            std::process::Command::new(path).spawn()
                        } else {
                            // AUMID — packaged (...!App) or auto-generated ({GUID})
                            std::process::Command::new("explorer.exe")
                                .arg(format!("shell:AppsFolder\\{path}"))
                                .spawn()
                        };
                    }

                    // ui.unwrap().window().set_minimized(true);
                    sfx.item();
                } else {
                    match item.label.as_str() {
                        "cigs" => sfx.item(),
                        "no item" => sfx.equip(),
                        _ => {}
                    }
                }
            }
        }
    });

    app.on_quit(move || {
        let __ = slint::quit_event_loop();
    });

    app.on_minimise({
        let ui = ui.as_weak();
        move || {
            ui.unwrap().window().set_minimized(true);
        }
    });

    sfx.start();

    ui.run()?;
    Ok(())
}