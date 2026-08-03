export namespace LauncherMeta {
	export namespace Manifest {
		export namespace v0 {
			export type LoaderVersion = {
				id: string
				url: string
				stable: boolean
			}

			export type GameVersionEntry = {
				id: string
				stable: boolean
				versionGroup?: string
				loaders: LoaderVersion[]
			}

			export type VersionGroup = {
				id: string
				loaders: LoaderVersion[]
			}

			export type Manifest = {
				gameVersions: GameVersionEntry[]
				versionGroups?: VersionGroup[]
			}
		}
	}
}
