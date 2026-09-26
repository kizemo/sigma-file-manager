use sigma_file_manager_lib::picker_state::{PickerHandle, PickerRegistry};
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
    let fake = PickerHandle(uuid::Uuid::new_v4());
    let result = reg.update_folder(&fake, PathBuf::from("C:/"));
    assert!(result.is_err());
}
