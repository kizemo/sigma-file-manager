// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

use sigma_file_manager_lib::picker::SigmaPicker;
use sigma_file_manager_lib::picker_state::PickerRegistry;
use std::path::PathBuf;

#[test]
fn registry_starts_empty() {
    let reg = PickerRegistry::new();
    assert_eq!(reg.count(), 0);
}

#[test]
fn register_assigns_handle_and_stores_folder() {
    let reg = PickerRegistry::new();
    let handle = reg.register(PathBuf::from("C:/Users"));
    assert_eq!(reg.count(), 1);
    assert_eq!(reg.current_folder(&handle).unwrap(), PathBuf::from("C:/Users"));
}

#[test]
fn update_folder_records_new_value_and_timestamp() {
    let reg = PickerRegistry::new();
    let handle = reg.register(PathBuf::from("C:/Users"));
    let before = reg.last_set_at(&handle).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    reg.update_folder(&handle, PathBuf::from("C:/Users/Documents")).unwrap();
    assert_eq!(reg.current_folder(&handle).unwrap(), PathBuf::from("C:/Users/Documents"));
    assert!(reg.last_set_at(&handle).unwrap() > before);
}

#[test]
fn unregister_removes_handle() {
    let reg = PickerRegistry::new();
    let handle = reg.register(PathBuf::from("C:/"));
    reg.unregister(&handle);
    assert_eq!(reg.count(), 0);
    assert!(reg.current_folder(&handle).is_none());
}

#[test]
fn update_folder_on_unknown_handle_returns_error() {
    let reg = PickerRegistry::new();
    // Forging a stale handle via the public API: register then unregister leaves
    // a structurally-valid PickerHandle that no longer maps to any state.
    let stale = reg.register(PathBuf::from("C:/"));
    reg.unregister(&stale);
    let result = reg.update_folder(&stale, PathBuf::from("C:/Windows"));
    assert!(result.is_err());
}

#[test]
fn sigma_picker_stores_initial_folder() {
    let picker = SigmaPicker::new(PathBuf::from("C:/foo")).unwrap();
    assert_eq!(picker.current_folder(), PathBuf::from("C:/foo").as_path());
}

#[test]
fn sigma_picker_hwnd_is_none_before_show() {
    let picker = SigmaPicker::new(PathBuf::from("C:/foo")).unwrap();
    assert!(picker.hwnd().is_none());
}

// COM smoke test — only meaningful on Windows with a display.
// In headless CI the underlying CoCreateInstance may fail; the test just
// verifies the call does not panic and returns a Result.
#[cfg(windows)]
#[test]
fn sigma_picker_creates_real_ifiledialog() {
    use sigma_file_manager_lib::picker::SigmaPicker;
    // Don't assert success — real GUI dialog creation may fail in headless CI.
    let result = SigmaPicker::new_com(PathBuf::from("C:/Users"));
    let _ = result;
}
