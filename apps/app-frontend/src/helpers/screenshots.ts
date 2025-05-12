import { invoke } from '@tauri-apps/api/core'

export type Screenshot = {
  path: string
  creation_date: string
}

export async function getAllProfileScreenshots(profilePath: string): Promise<Screenshot[]> {
  return await invoke<Screenshot[]>('plugin:screenshots|get_all_profile_screenshots', {
    path: profilePath,
  })
}

export async function deleteProfileScreenshot(
  profilePath: string,
  screenshot: Screenshot,
): Promise<boolean> {
  return await invoke<boolean>('plugin:screenshots|delete_profile_screenshot', {
    path: profilePath,
    screenshot,
  })
}

export async function openProfileScreenshot(
  profilePath: string,
  screenshot: Screenshot,
): Promise<boolean> {
  return await invoke<boolean>('plugin:screenshots|open_profile_screenshot', {
    path: profilePath,
    screenshot,
  })
}

export async function getScreenshotData(
  profilePath: string,
  screenshot: Screenshot,
): Promise<string | undefined> {
  return await invoke<string | undefined>('plugin:screenshots|get_screenshot_data', {
    path: profilePath,
    screenshot,
  })
}

export function getScreenshotFileName(path: string | undefined) {
  if (!path) return 'Untitled'
  return path.split('/').pop()!
}
