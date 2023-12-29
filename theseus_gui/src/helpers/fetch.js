import { ofetch } from 'ofetch'
import { handleError } from '@/store/state.js'
import { getVersion } from '@tauri-apps/api/app'

export const useFetch = async (url, item, isSilent, signal) => {
  try {
    const version = await getVersion()

    return await ofetch(url, {
      headers: { 'User-Agent': `modrinth/theseus/${version} (support@modrinth.com)` },
      signal: signal,
    })
  } catch (err) {
    if (err.message.startsWith('Fetch is aborted') || err.message.startsWith('Request signal is aborted')) {
      throw err;
    }

    if (!isSilent) {
      handleError({ message: `Error fetching ${item}` })
    }
    console.error(err)
  }
}
