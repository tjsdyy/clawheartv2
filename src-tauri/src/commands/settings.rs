// IPC: 设置 / 主题
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub start_with_system: bool,
    pub compact_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "paper".into(),
            language: "zh".into(),
            start_with_system: false,
            compact_mode: false,
        }
    }
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppResult<Settings> {
    let ui = state.ui.lock().unwrap();
    Ok(Settings {
        theme: ui.theme.clone(),
        language: ui.language.clone(),
        ..Settings::default()
    })
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: Settings) -> AppResult<()> {
    {
        let mut ui = state.ui.lock().unwrap();
        ui.theme = settings.theme.clone();
        ui.language = settings.language.clone();
    }

    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let _ = crate::storage::queries::settings::set(db, "theme", &settings.theme);
            let _ = crate::storage::queries::settings::set(db, "language", &settings.language);
        }
    }
    let _ = settings;
    Ok(())
}

#[tauri::command]
pub fn set_theme(state: State<AppState>, theme: String) -> AppResult<()> {
    {
        let mut ui = state.ui.lock().unwrap();
        ui.theme = theme.clone();
    }
    #[cfg(feature = "storage")]
    {
        if let Some(db) = &state.db {
            let _ = crate::storage::queries::settings::set(db, "theme", &theme);
        }
    }
    let _ = theme;
    Ok(())
}
