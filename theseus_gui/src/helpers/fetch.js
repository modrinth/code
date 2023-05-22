import { ofetch } from 'ofetch'
import { handleError } from '@/store/state.js'

export const useFetch = async (url, item) => {
  try {
    return await ofetch(url)
  } catch (err) {
    handleError({ message: `Error fetching ${item}` })
  }
}
