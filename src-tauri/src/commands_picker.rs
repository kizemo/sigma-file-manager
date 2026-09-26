// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

//! Tauri command surface for the native file picker dialog (`SigmaPicker`).
//!
//! ## Threading model
//!
//! `SigmaPicker` holds a Windows `IFileOpenDialog` COM interface, which is
//! `!Send + !Sync`. The live picker instances MUST live on a single STA
//! apartment thread. To keep the Tauri command surface `async`, every
//! command here is a thin pass-through to a long-lived worker thread that
//! owns the pickers and the registry.
//!
//! On non-Windows targets the picker is unavailable (no `IFileOpenDialog`),
//! so the command bodies return `Err` instead of dispatching to the worker.
//! The command functions themselves are present on every target so the
//! `invoke_handler!` block in `lib.rs` does not need `#[cfg(windows)]`.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::picker::SigmaPicker;
use crate::picker_state::{PickerHandle, PickerRegistry};

// ---------- Windows-only worker thread plumbing ----------

#[cfg(windows)]
mod worker {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use tokio::sync::{mpsc, oneshot};

    use super::{PickerHandle, PickerRegistry, SigmaPicker};

    /// Commands sent from Tauri command threads to the STA worker thread.
    pub enum PickerCmd {
        Open {
            folder: PathBuf,
            reply: oneshot::Sender<Result<PickerHandle, String>>,
        },
        SetFolder {
            handle: PickerHandle,
            folder: PathBuf,
            reply: oneshot::Sender<Result<(), String>>,
        },
        CloseAndGetResult {
            handle: PickerHandle,
            reply: oneshot::Sender<Result<Option<PathBuf>, String>>,
        },
    }

    /// Live picker instances + the picker registry. Lives only on the worker thread.
    struct WorkerState {
        pickers: HashMap<PickerHandle, SigmaPicker>,
        registry: PickerRegistry,
    }

    /// Cloneable handle to the STA worker thread.
    ///
    /// The `mpsc::UnboundedSender` is `Send + Sync`, so `PickerWorker` is
    /// cheaply cloneable and can be stored in Tauri's app state.
    #[derive(Clone)]
    pub struct PickerWorker {
        sender: mpsc::UnboundedSender<PickerCmd>,
    }

    impl PickerWorker {
        /// Spawn the dedicated STA worker thread. The thread runs `SigmaPicker`
        /// COM calls; the picker instances never leave this thread.
        pub fn construct() -> Self {
            Self::spawn()
        }

        /// Spawn the dedicated STA worker thread. The thread runs `SigmaPicker`
        /// COM calls; the picker instances never leave this thread.
        pub fn spawn() -> Self {
            let (tx, mut rx) = mpsc::unbounded_channel::<PickerCmd>();
            std::thread::Builder::new()
                .name("sigma-picker-worker".into())
                .spawn(move || {
                    let mut state = WorkerState {
                        pickers: HashMap::new(),
                        registry: PickerRegistry::new(),
                    };
                    while let Some(cmd) = rx.blocking_recv() {
                        match cmd {
                            PickerCmd::Open { folder, reply } => {
                                let result = (|| -> Result<PickerHandle, String> {
                                    let picker = SigmaPicker::new_com(folder.clone())
                                        .map_err(|e| e.to_string())?;
                                    let handle = state.registry.register(folder);
                                    state.pickers.insert(handle, picker);
                                    Ok(handle)
                                })();
                                let _ = reply.send(result);
                            }
                            PickerCmd::SetFolder { handle, folder, reply } => {
                                let result = (|| -> Result<(), String> {
                                    let picker = state
                                        .pickers
                                        .get_mut(&handle)
                                        .ok_or_else(|| "handle not found".to_string())?;
                                    picker
                                        .set_folder(folder.clone())
                                        .map_err(|e| e.to_string())?;
                                    state
                                        .registry
                                        .update_folder(&handle, folder)
                                        .map_err(|e| e.to_string())?;
                                    Ok(())
                                })();
                                let _ = reply.send(result);
                            }
                            PickerCmd::CloseAndGetResult { handle, reply } => {
                                let result = (|| -> Result<Option<PathBuf>, String> {
                                    let picker = state
                                        .pickers
                                        .get_mut(&handle)
                                        .ok_or_else(|| "handle not found".to_string())?;
                                    // `get_result` errors when the user cancelled or the
                                    // shell item conversion failed. Cancellation is the
                                    // expected "user dismissed the dialog" path, so map
                                    // any error to `None` — callers can't distinguish
                                    // cancel from shell errors here. If that distinction
                                    // becomes important, surface the HRESULT instead.
                                    let path = picker.get_result().ok();
                                    picker.close();
                                    state.pickers.remove(&handle);
                                    state.registry.unregister(&handle);
                                    Ok(path)
                                })();
                                let _ = reply.send(result);
                            }
                        }
                    }
                })
                .expect("failed to spawn sigma-picker-worker thread");
            Self { sender: tx }
        }

        pub async fn open(&self, folder: PathBuf) -> Result<PickerHandle, String> {
            let (reply, rx) = oneshot::channel();
            self.sender
                .send(PickerCmd::Open { folder, reply })
                .map_err(|_| "picker worker thread is dead".to_string())?;
            rx.await
                .map_err(|_| "picker worker reply dropped".to_string())?
        }

        pub async fn set_folder(
            &self,
            handle: PickerHandle,
            folder: PathBuf,
        ) -> Result<(), String> {
            let (reply, rx) = oneshot::channel();
            self.sender
                .send(PickerCmd::SetFolder { handle, folder, reply })
                .map_err(|_| "picker worker thread is dead".to_string())?;
            rx.await
                .map_err(|_| "picker worker reply dropped".to_string())?
        }

        pub async fn close_and_get_result(
            &self,
            handle: PickerHandle,
        ) -> Result<Option<PathBuf>, String> {
            let (reply, rx) = oneshot::channel();
            self.sender
                .send(PickerCmd::CloseAndGetResult { handle, reply })
                .map_err(|_| "picker worker thread is dead".to_string())?;
            rx.await
                .map_err(|_| "picker worker reply dropped".to_string())?
        }
    }
}

#[cfg(windows)]
pub use worker::PickerWorker;

#[cfg(not(windows))]
#[derive(Clone)]
pub struct PickerWorker;

#[cfg(not(windows))]
impl PickerWorker {
    pub fn construct() -> Self {
        Self
    }
}

// ---------- Public construction entry point ----------

/// Construct the picker worker to be installed via `app.manage(...)`.
///
/// On Windows this spawns the dedicated STA worker thread. On non-Windows
/// the worker has no work to do (there is no COM picker to drive); we still
/// need *something* `Send + Sync + 'static` to register so the command
/// surface compiles. The command bodies short-circuit to `Err` on non-Windows
/// before touching this state, so the no-op stub is never read.
pub fn build_picker_worker() -> PickerWorker {
    PickerWorker::construct()
}

// ---------- Tauri command surface ----------

#[tauri::command]
pub async fn picker_open(folder: String, app: AppHandle) -> Result<String, String> {
    #[cfg(windows)]
    {
        let worker: PickerWorker = app.state::<PickerWorker>().inner().clone();
        let handle = worker.open(PathBuf::from(folder)).await?;
        Ok(handle.as_uuid().to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = (folder, app);
        Err("picker_open is only supported on Windows".to_string())
    }
}

#[tauri::command]
pub async fn picker_set_folder(
    handle: String,
    folder: String,
    app: AppHandle,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let uuid = uuid::Uuid::parse_str(&handle).map_err(|e| e.to_string())?;
        let h = PickerHandle::from_uuid(uuid);
        let worker: PickerWorker = app.state::<PickerWorker>().inner().clone();
        worker.set_folder(h, PathBuf::from(folder)).await
    }
    #[cfg(not(windows))]
    {
        let _ = (handle, folder, app);
        Err("picker_set_folder is only supported on Windows".to_string())
    }
}

#[tauri::command]
pub async fn picker_close(handle: String, app: AppHandle) -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        let uuid = uuid::Uuid::parse_str(&handle).map_err(|e| e.to_string())?;
        let h = PickerHandle::from_uuid(uuid);
        let worker: PickerWorker = app.state::<PickerWorker>().inner().clone();
        let result = worker.close_and_get_result(h).await?;
        Ok(result.map(|p| p.to_string_lossy().into_owned()))
    }
    #[cfg(not(windows))]
    {
        let _ = (handle, app);
        Err("picker_close is only supported on Windows".to_string())
    }
}