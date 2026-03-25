use std::sync::Mutex;

#[derive(Default)]
pub struct UpdateState {
    pub pending_update: Mutex<Option<tauri_plugin_updater::Update>>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started { content_length: Option<u64> },
    #[serde(rename_all = "camelCase")]
    Progress { chunk_length: usize, downloaded: u64 },
    Finished,
}

mod commands {
    use super::{DownloadEvent, UpdateInfo, UpdateState};
    use tauri::AppHandle;
    use tauri_plugin_updater::UpdaterExt;

    /// Check for an available update. Returns update info if found, or null if already up to date.
    #[tauri::command]
    pub async fn check_update(
        app: AppHandle,
        state: tauri::State<'_, UpdateState>,
    ) -> Result<Option<UpdateInfo>, String> {
        let updater = app.updater().map_err(|e| e.to_string())?;
        let current_version = app.package_info().version.to_string();

        match updater.check().await {
            Ok(Some(update)) => {
                let info = UpdateInfo {
                    version: update.version.clone(),
                    current_version,
                    notes: update.body.clone(),
                };
                *state.pending_update.lock().unwrap() = Some(update);
                Ok(Some(info))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Download and install the pending update, streaming progress via a channel.
    #[tauri::command]
    pub async fn apply_update(
        app: AppHandle,
        state: tauri::State<'_, UpdateState>,
        on_event: tauri::ipc::Channel<DownloadEvent>,
    ) -> Result<(), String> {
        let update = state
            .pending_update
            .lock()
            .unwrap()
            .take()
            .ok_or("No pending update. Call checkUpdate first.")?;

        let mut downloaded: u64 = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    if downloaded == 0 {
                        let _ = on_event.send(DownloadEvent::Started { content_length });
                    }
                    downloaded += chunk_length as u64;
                    let _ = on_event.send(DownloadEvent::Progress {
                        chunk_length,
                        downloaded,
                    });
                },
                || {
                    let _ = on_event.send(DownloadEvent::Finished);
                },
            )
            .await
            .map_err(|e| e.to_string())?;

        app.restart();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .manage(UpdateState::default())
        .invoke_handler(tauri::generate_handler![
            commands::check_update,
            commands::apply_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
