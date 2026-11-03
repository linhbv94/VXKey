import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { SurfaceType } from '../types';
const surfaces: readonly string[] = ['settings','tray','history','conversion','onboarding'];
export function initialSurface(): SurfaceType {
  const native = '__TAURI_INTERNALS__' in window;
  const label = native ? getCurrentWebviewWindow().label : new URLSearchParams(location.search).get('surface');
  return label && surfaces.includes(label) ? label as SurfaceType : 'settings';
}
