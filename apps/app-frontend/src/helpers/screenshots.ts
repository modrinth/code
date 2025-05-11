import {invoke} from "@tauri-apps/api/core";

export type Screenshot = {
    path: string;
    creation_date: string;
    data: string;
}

export async function getAllProfileScreenshots(path: string): Promise<Screenshot[]> {
    return await invoke('plugin:screenshots|get_all_profile_screenshots', { path })
}

export async function deleteScreenshotFile(screenshot: Screenshot): Promise<boolean> {
    return await invoke('plugin:screenshots|delete_screenshot', {path: screenshot.path})
}

export async function renameScreenshotFile(screenshot: Screenshot, new_filename: string): Promise<boolean> {
    return await invoke('plugin:screenshots|rename_screenshot', {path: screenshot.path, new_filename})
}