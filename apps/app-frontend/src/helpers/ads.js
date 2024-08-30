import { invoke } from '@tauri-apps/api/core'

export async function init_ads_window(x, y, width, height, overrideShown) {
  return await invoke('plugin:ads|init_ads_window', { x, y, width, height, overrideShown })
}

export async function show_ads_window() {
  return await invoke('plugin:ads|show_ads_window')
}

export async function hide_ads_window(reset) {
  return await invoke('plugin:ads|hide_ads_window', { reset })
}
