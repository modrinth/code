import {invoke} from "@tauri-apps/api/core";

type Screenshot = {
    filename: string;
    creation_date: string;
}

export async function getAllProfileScreenshots(path: string): Promise<Screenshot[]> {
    return await invoke('plugin:screenshots|get_all_profile_screenshots', { path })
}