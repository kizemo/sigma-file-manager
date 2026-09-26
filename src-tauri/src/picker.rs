// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows::Win32::Foundation::HWND;

/// Lightweight wrapper around the Windows `IFileOpenDialog` COM object.
///
/// On non-Windows targets the dialog field is absent and `new_com` is
/// unavailable — the rest of the API still works as a folder-tracking stub
/// so registry code can compile and test cross-platform.
pub struct SigmaPicker {
    initial_folder: PathBuf,
    #[cfg(windows)]
    dialog_hwnd: Option<HWND>,
    #[cfg(windows)]
    dialog: Option<windows::Win32::UI::Shell::IFileOpenDialog>,
}

impl SigmaPicker {
    /// Construct a picker stub without touching COM. Used by tests and as a
    /// fallback when full COM wiring is unavailable (non-Windows, or COM init
    /// failed). For the real `IFileOpenDialog`, use [`SigmaPicker::new_com`].
    pub fn new(folder: PathBuf) -> Result<Self, String> {
        Ok(Self {
            initial_folder: folder,
            #[cfg(windows)]
            dialog_hwnd: None,
            #[cfg(windows)]
            dialog: None,
        })
    }

    /// Initial folder the dialog was opened with.
    pub fn current_folder(&self) -> &Path {
        &self.initial_folder
    }

    /// HWND associated with this picker.
    ///
    /// On Windows this is populated by [`SigmaPicker::show`] with the owner
    /// HWND passed in. Until `show` is called it is `None`. Task 5 may switch
    /// to the actual dialog HWND (via `FindWindowEx`) once the threading
    /// model is in place.
    #[cfg(windows)]
    pub fn hwnd(&self) -> Option<HWND> {
        self.dialog_hwnd
    }

    #[cfg(not(windows))]
    pub fn hwnd(&self) -> Option<()> {
        None
    }

    /// Release the dialog. Dropping the COM object ends the modal loop if it
    /// is still running.
    pub fn close(&mut self) {
        #[cfg(windows)]
        {
            self.dialog = None;
        }
    }
}

#[cfg(windows)]
impl SigmaPicker {
    /// Create a real `IFileOpenDialog` initialized to `initial_folder`.
    ///
    /// **Caller must own the thread.** `IFileDialog` requires
    /// `COINIT_APARTMENTTHREADED`; this call will return `RPC_E_CHANGED_MODE`
    /// (0x80010106) if the current thread already has COM initialized in a
    /// different mode. Task 5 wires a dedicated worker thread; for now this
    /// function expects to be called from a fresh STA thread.
    ///
    /// `S_FALSE` (returned when COM was already initialized as STA on this
    /// thread) is treated as success and does not error out, since
    /// `HRESULT::ok()` treats any non-negative code as success.
    pub fn new_com(initial_folder: PathBuf) -> windows::core::Result<Self> {
        unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
            )
            .ok()?;

            let dialog: windows::Win32::UI::Shell::IFileOpenDialog =
                windows::Win32::System::Com::CoCreateInstance(
                    &windows::Win32::UI::Shell::FileOpenDialog,
                    None,
                    windows::Win32::System::Com::CLSCTX_INPROC_SERVER,
                )?;

            // SetFolder takes the folder IShellItem itself (not a child file).
            let item: windows::Win32::UI::Shell::IShellItem =
                shell_item_from_path(&initial_folder)?;
            dialog.SetFolder(&item)?;

            Ok(Self {
                initial_folder,
                dialog_hwnd: None,
                dialog: Some(dialog),
            })
        }
    }

    /// Show the dialog modally. Blocks until the user dismisses it.
    ///
    /// `owner` is cached as `dialog_hwnd` — the real dialog HWND is not
    /// queryable through `IFileDialog::GetWindow` reliably across versions,
    /// so for Task 4 we treat the owner HWND as the focus target. Task 5
    /// may upgrade this to `FindWindowExW` against the dialog title.
    pub fn show(&mut self, owner: HWND) -> windows::core::Result<()> {
        let dialog = self
            .dialog
            .as_ref()
            .ok_or_else(windows::core::Error::from_win32)?;
        unsafe {
            dialog.Show(owner)?;
        }
        self.dialog_hwnd = Some(owner);
        Ok(())
    }

    /// Update the dialog's current folder. Works against a live dialog.
    ///
    /// Updates `initial_folder` to reflect the new value so registry state
    /// stays consistent with the dialog.
    pub fn set_folder(&mut self, folder: PathBuf) -> windows::core::Result<()> {
        let dialog = self
            .dialog
            .as_ref()
            .ok_or_else(windows::core::Error::from_win32)?;
        let item: windows::Win32::UI::Shell::IShellItem = shell_item_from_path(&folder)?;
        unsafe {
            dialog.SetFolder(&item)?;
        }
        self.initial_folder = folder;
        Ok(())
    }

    /// Extract the user-selected path as a `PathBuf`.
    ///
    /// Call only after [`show`](Self::show) returns successfully. Returns an
    /// error if the dialog was cancelled (no result) or if the path
    /// conversion fails.
    pub fn get_result(&self) -> windows::core::Result<PathBuf> {
        let dialog = self
            .dialog
            .as_ref()
            .ok_or_else(windows::core::Error::from_win32)?;
        let item: windows::Win32::UI::Shell::IShellItem = unsafe { dialog.GetResult() }?;
        let pwstr =
            unsafe { item.GetDisplayName(windows::Win32::UI::Shell::SIGDN_FILESYSPATH) }?;
        // PWSTR is a *mut u16 pointing at a null-terminated UTF-16 string
        // owned by the shell item. `to_string` walks until the NUL terminator.
        let path =
            unsafe { pwstr.to_string() }.map_err(|_| windows::core::Error::from_win32())?;
        Ok(PathBuf::from(path))
    }
}

#[cfg(windows)]
fn shell_item_from_path(
    path: &Path,
) -> windows::core::Result<windows::Win32::UI::Shell::IShellItem> {
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        windows::Win32::UI::Shell::SHCreateItemFromParsingName(
            windows::core::PCWSTR(wide.as_ptr()),
            None,
        )
    }
}
