
// Wrapper around convertFileSrc, which converts a filepath to a loadable URL

import { invoke } from "@tauri-apps/api"
import { convertFileSrc } from "@tauri-apps/api/tauri"

// This converts a relative cached filepath (ie: 'icons/1234.png') to a loadable URL by appending the cache directory first
export function convertCachedFileSrc(cacheDir,src) {
  return convertFileSrc(cacheDir + "/" + src)
}

export async function getCacheDir() {
  return await invoke('plugin:utils|get_cache_path', { })
}

