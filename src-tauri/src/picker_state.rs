// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PickerHandle(Uuid);

impl PickerHandle {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug)]
pub struct PickerState {
    pub handle: PickerHandle,
    pub current_folder: PathBuf,
    pub last_set_at: Instant,
}

pub struct PickerRegistry {
    inner: Arc<Mutex<HashMap<PickerHandle, PickerState>>>,
}

impl PickerRegistry {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn register(&self, folder: PathBuf) -> PickerHandle {
        let handle = PickerHandle::new();
        let state = PickerState { handle, current_folder: folder, last_set_at: Instant::now() };
        self.inner.lock().expect("picker_state mutex poisoned").insert(handle, state);
        handle
    }

    pub fn update_folder(&self, handle: &PickerHandle, folder: PathBuf) -> Result<(), &'static str> {
        let mut map = self.inner.lock().expect("picker_state mutex poisoned");
        let state = map.get_mut(handle).ok_or("handle not found")?;
        state.current_folder = folder;
        state.last_set_at = Instant::now();
        Ok(())
    }

    pub fn unregister(&self, handle: &PickerHandle) {
        self.inner.lock().expect("picker_state mutex poisoned").remove(handle);
    }

    pub fn current_folder(&self, handle: &PickerHandle) -> Option<PathBuf> {
        self.inner.lock().expect("picker_state mutex poisoned").get(handle).map(|s| s.current_folder.clone())
    }

    pub fn last_set_at(&self, handle: &PickerHandle) -> Option<Instant> {
        self.inner.lock().expect("picker_state mutex poisoned").get(handle).map(|s| s.last_set_at)
    }

    pub fn count(&self) -> usize {
        self.inner.lock().expect("picker_state mutex poisoned").len()
    }
}

impl Default for PickerRegistry {
    fn default() -> Self { Self::new() }
}
