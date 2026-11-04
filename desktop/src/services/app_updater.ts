import { wrapUpdate } from './update_handle';
import { useEffect, useSyncExternalStore } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { createUpdateController } from './update_controller';
import packageInfo from '../../package.json';

export const appVersion = packageInfo.version;
const controller = createUpdateController({
  enabled: isTauri() && import.meta.env.VITE_VXKEY_TEST_APP !== '1' && !import.meta.env.DEV && getCurrentWindow().label === 'settings',
  currentVersion: appVersion,
  check: async (options) => {
    const update = await check(options);
    if (!update) return null;
    return wrapUpdate(update, () => invoke('vxkey_prepare_update'));
  },
  relaunch,
  beforeInstall: async () => {
    if (!window.dispatchEvent(new Event('vx-before-update', { cancelable: true }))) throw new Error('operation_busy_or_unsaved');
  },
});
export const { checkForUpdates, installUpdate, restartAfterUpdate, dismissUpdateNotice } = controller;

export function useAppUpdater() {
  const state = useSyncExternalStore(controller.subscribe, controller.snapshot);
  // Startup and Settings share one store, including under React StrictMode.
  useEffect(controller.start, []);
  return state;
}
