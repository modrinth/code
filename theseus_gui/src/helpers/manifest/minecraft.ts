export interface Version {
  id: string
  type_: 'Release' | 'Snapshot' | 'OldAlpha' | 'OldBeta'
  url: string
  time: number
  release_time: number
  sha1: string
  compliance_level: number
  assets_index_url?: string
  assets_index_sha1?: string
}

export interface LatestVersion {
  release: string
  snapshot: string
}

export interface VersionManifest {
  latest: LatestVersion
  versions: Version[]
}
