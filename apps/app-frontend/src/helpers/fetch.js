import { fetch } from '@tauri-apps/plugin-http'
import { handleError } from '@/store/state.js'
import { getVersion } from '@tauri-apps/api/app'

export const useFetch = async (url, item, isSilent) => {
  try {
    const version = await getVersion()
    return await fetch(url, {
      method: 'GET',
      headers: { 'User-Agent': `modrinth/theseus/${version} (support@modrinth.com)` },
    })
  } catch (err) {
    if (!isSilent) {
      handleError({ message: `Error fetching ${item}` })
    }
    console.error(err)
  }
}
