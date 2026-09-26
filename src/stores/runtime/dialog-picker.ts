// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useDialogPickerStore = defineStore('dialogPicker', () => {
  const activeHandle = ref<string | null>(null);
  const lastKnownFolder = ref<string | null>(null);

  async function open(initialFolder: string) {
    const handle = await invoke<string>('picker_open', { folder: initialFolder });
    activeHandle.value = handle;
    lastKnownFolder.value = initialFolder;
  }

  async function setFolder(folder: string) {
    if (!activeHandle.value) return;
    await invoke('picker_set_folder', { handle: activeHandle.value, folder });
    lastKnownFolder.value = folder;
  }

  async function close(): Promise<string | null> {
    if (!activeHandle.value) return null;
    const selected = await invoke<string | null>('picker_close', { handle: activeHandle.value });
    activeHandle.value = null;
    lastKnownFolder.value = null;
    return selected;
  }

  return {
    activeHandle,
    lastKnownFolder,
    open,
    setFolder,
    close,
  };
});