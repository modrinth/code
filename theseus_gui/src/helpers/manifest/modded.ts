export interface Manifest {
  game_versions: ModdedVersion[]
}

export interface ModdedVersion {
  id: string
  stable: boolean
  loaders: LoaderVersion[]
}

export interface LoaderVersion {
  id: string
  url: string
  stable: boolean
}
