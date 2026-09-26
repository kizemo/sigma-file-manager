// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

//! Tests for the picker worker-thread plumbing (`commands_picker`).
//!
//! The Tauri command wrappers themselves are thin pass-throughs that need a
//! `tauri::AppHandle`; we test the underlying `PickerWorker` directly so the
//! assertions don't require a Tauri app fixture.

#[cfg(windows)]
mod windows_tests {
    use std::path::PathBuf;

    use sigma_file_manager_lib::commands_picker::PickerWorker;
    use sigma_file_manager_lib::picker_state::PickerHandle;

    #[tokio::test]
    async fn picker_worker_round_trips_open_and_close() {
        // Spawning the worker must always succeed (it returns immediately;
        // the thread itself spawns asynchronously).
        let worker = PickerWorker::spawn();

        // `Open` calls `SigmaPicker::new_com`, which exercises real COM.
        // In a headless CI environment `CoCreateInstance` may fail with
        // some shell error; accept that as a known limitation and only
        // assert when COM is reachable.
        let open = worker.open(PathBuf::from("C:/Users")).await;
        if let Ok(handle) = open {
            // Closing the picker we just opened should clean up without error.
            // The result is `None` because `show()` was never called, so the
            // COM dialog has no selected result.
            let close = worker.close_and_get_result(handle).await;
            assert!(
                close.is_ok(),
                "close_and_get_result on a freshly-opened picker should succeed: {close:?}"
            );
        }
    }

    #[tokio::test]
    async fn picker_worker_close_unknown_handle_errors() {
        let worker = PickerWorker::spawn();
        let stale = PickerHandle::new();
        let err = worker
            .close_and_get_result(stale)
            .await
            .expect_err("unknown handle must error");
        assert!(
            err.contains("handle not found"),
            "error must mention handle not found, got: {err}"
        );
    }

    #[tokio::test]
    async fn picker_worker_set_folder_unknown_handle_errors() {
        let worker = PickerWorker::spawn();
        let stale = PickerHandle::new();
        let err = worker
            .set_folder(stale, PathBuf::from("C:/Windows"))
            .await
            .expect_err("unknown handle must error");
        assert!(
            err.contains("handle not found"),
            "error must mention handle not found, got: {err}"
        );
    }
}