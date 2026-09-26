// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

pub struct SigmaPicker {
    initial_folder: PathBuf,
    #[cfg(windows)]
    dialog_hwnd: Option<HWND>,
}

impl SigmaPicker {
    pub fn new(folder: PathBuf) -> Result<Self, String> {
        // Real COM init comes in Task 4. For now: just record the folder.
        Ok(Self {
            initial_folder: folder,
            #[cfg(windows)]
            dialog_hwnd: None,
        })
    }

    pub fn current_folder(&self) -> &Path {
        &self.initial_folder
    }

    #[cfg(windows)]
    pub fn hwnd(&self) -> Option<HWND> {
        self.dialog_hwnd
    }

    #[cfg(not(windows))]
    pub fn hwnd(&self) -> Option<()> {
        None
    }

    pub fn close(&mut self) {
        // No-op until Task 4 wires real IFileDialog.
    }
}