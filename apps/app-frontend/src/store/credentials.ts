import { defineStore } from 'pinia'
import { ref } from 'vue'
import { get as getCreds } from '../helpers/mr_auth.js'
import { get_user } from '../helpers/cache.js'
import { handleError } from '../store/state'

export interface User {
  id: string
  username: string
  avatar_url: string
  bio: string
  created: Date
  role: string
  badges: number
}

export interface Credentials {
  session: string
  expires: Date
  user_id: string
  active: boolean
  user: User | null
}

export type SessionToken = Credentials['session']

export const useCredentialsStore = defineStore('credentials', () => {
  const credentials = ref<Credentials | null>(null)
  const error = ref<Error | null>(null)

  async function fetchCredentials() {
    try {
      const creds = await getCreds()
      if (creds && creds.user_id) {
        creds.user = await get_user(creds.user_id)
      }
      credentials.value = creds
    } catch (err) {
      handleError(err)
      error.value = err as Error
    }
  }

  function clearCredentials() {
    credentials.value = null
    error.value = null
  }

  return {
    credentials,
    error,
    fetchCredentials,
    clearCredentials,
  }
})
