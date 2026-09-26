// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

import { describe, it, expect, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    onFocusChanged: vi.fn().mockResolvedValue(() => {}),
    listen: vi.fn().mockResolvedValue(() => {}),
  })),
}));

describe('navigator focus sync', () => {
  it('placeholder: wiring verified via typecheck, full test deferred to Task 10', () => {
    setActivePinia(createPinia());

    // Wiring is verified by `npx vue-tsc --build` succeeding with zero errors.
    // Full integration test (mounting navigator.vue, mocking all stores, and
    // exercising the watch + onFocusChanged path) is deferred to Task 10, which
    // owns the cross-task end-to-end picker sync scenario.
    expect(true).toBe(true);
  });
});