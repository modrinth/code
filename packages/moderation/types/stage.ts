import type { Action } from './actions'

export interface Stage {
  title: string
  guidance_url: string
  actions: Action[]
}
