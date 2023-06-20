import { ofetch } from 'ofetch'
import { handleError } from '@/store/state.js'

export const useFetch = async (url, item) => {
  try {
    return await ofetch(url, {
      headers: { 'User-Agent': 'modrinth/theseus (support@modrinth.com)' },
    })
  } catch (err) {
    handleError({ message: `Error fetching ${item}` })
    console.error(err)
  }
}
