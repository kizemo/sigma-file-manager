// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

import { describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

const { currentTabState } = vi.hoisted(() => ({
  currentTabState: { value: null as { path: string } | null },
}));

vi.mock('@/stores/storage/workspaces', () => ({
  useWorkspacesStore: () => ({
    get currentTab() {
      return currentTabState.value;
    },
  }),
}));

describe('useCurrentNavigatorPath', () => {
  it('returns null when there is no active tab', async () => {
    setActivePinia(createPinia());
    currentTabState.value = null;

    const { useCurrentNavigatorPath } = await import('@/composables/use-current-navigator-path');
    const path = useCurrentNavigatorPath();

    expect(path.value).toBeNull();
  });

  it('returns the active tab path when a tab is active', async () => {
    setActivePinia(createPinia());
    currentTabState.value = { path: 'C:/Projects' };

    const { useCurrentNavigatorPath } = await import('@/composables/use-current-navigator-path');
    const path = useCurrentNavigatorPath();

    expect(path.value).toBe('C:/Projects');
  });
});