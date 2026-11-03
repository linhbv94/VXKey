import type { Update } from '@tauri-apps/plugin-updater';
export function wrapUpdate(update: Update, prepareRestart: () => Promise<unknown>) {
  return { version: update.version, close: () => update.close(),
    downloadAndInstall: async (progress: Parameters<Update['downloadAndInstall']>[0]) => {
      await update.download(progress);
      await update.install();
      await prepareRestart();
    } };
}
