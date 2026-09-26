// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { useDialogPickerStore } from '../dialog-picker';
import { invoke } from '@tauri-apps/api/core';

describe('useDialogPickerStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.mocked(invoke).mockReset();
  });

  it('starts with no active handle', () => {
    const store = useDialogPickerStore();
    expect(store.activeHandle).toBeNull();
  });

  it('open() invokes picker_open and stores handle', async () => {
    vi.mocked(invoke).mockResolvedValue('fake-uuid-1');
    const store = useDialogPickerStore();
    await store.open('C:/foo');
    expect(invoke).toHaveBeenCalledWith('picker_open', { folder: 'C:/foo' });
    expect(store.activeHandle).toBe('fake-uuid-1');
    expect(store.lastKnownFolder).toBe('C:/foo');
  });

  it('setFolder() invokes picker_set_folder with active handle', async () => {
    vi.mocked(invoke).mockResolvedValue('fake-uuid-2');
    const store = useDialogPickerStore();
    await store.open('C:/foo');
    vi.mocked(invoke).mockClear();
    await store.setFolder('C:/bar');
    expect(invoke).toHaveBeenCalledWith('picker_set_folder', { handle: 'fake-uuid-2', folder: 'C:/bar' });
    expect(store.lastKnownFolder).toBe('C:/bar');
  });

  it('close() invokes picker_close and clears state', async () => {
    vi.mocked(invoke).mockResolvedValue('fake-uuid-3');
    const store = useDialogPickerStore();
    await store.open('C:/foo');
    vi.mocked(invoke).mockClear();
    vi.mocked(invoke).mockResolvedValue('C:/foo/file.txt');
    await store.close();
    expect(invoke).toHaveBeenCalledWith('picker_close', { handle: 'fake-uuid-3' });
    expect(store.activeHandle).toBeNull();
  });
});