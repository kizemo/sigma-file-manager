// SPDX-License-Identifier: GPL-3.0-or-later
// License: GNU GPLv3 or later. See the license file in the project root for more information.
// Copyright © 2021 - present Aleksey Hoffman. All rights reserved.

import { computed, type ComputedRef } from 'vue';
import { useWorkspacesStore } from '@/stores/storage/workspaces';

/**
 * Returns the active navigator tab path as a `ComputedRef<string | null>`.
 *
 * The active tab is determined by the workspaces store's `currentTab` selector,
 * which resolves the current workspace → current tab group → current tab. When
 * no tab is active (empty store, pre-init, or all tabs closed), the composable
 * returns `null` so callers can distinguish "no path" from "empty string".
 */
export function useCurrentNavigatorPath(): ComputedRef<string | null> {
  const workspaces = useWorkspacesStore();
  return computed(() => workspaces.currentTab?.path ?? null);
}