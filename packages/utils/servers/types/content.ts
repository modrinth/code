export interface Mod {
  filename: string
  project_id: string | undefined
  version_id: string | undefined
  name: string | undefined
  version_number: string | undefined
  icon_url: string | undefined
  owner: string | undefined
  disabled: boolean
  installing: boolean
}

export type ContentType = 'mod' | 'plugin'
