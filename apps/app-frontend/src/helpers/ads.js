import { invoke } from '@tauri-apps/api/core'

export async function init_ads_window(overrideShown = false) {
	return await invoke('plugin:ads|init_ads_window', {
		overrideShown,
		dpr: window.devicePixelRatio,
	})
}

export async function show_ads_window() {
	return await invoke('plugin:ads|show_ads_window', { dpr: window.devicePixelRatio })
}

export async function hide_ads_window(reset) {
	return await invoke('plugin:ads|hide_ads_window', { reset })
}

export async function record_ads_click() {
	return await invoke('plugin:ads|record_ads_click')
}

export async function open_ads_link(path, origin) {
	return await invoke('plugin:ads|open_link', { path, origin })
}
