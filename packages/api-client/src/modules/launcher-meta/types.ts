export namespace LauncherMeta {
	export namespace Manifest {
		export namespace v0 {
			export type LoaderVersion = {
				id: string
				stable: boolean
			}

			export type GameVersionEntry = {
				id: string
				loaders: LoaderVersion[]
			}

			export type Manifest = {
				gameVersions: GameVersionEntry[]
			}
		}
	}
}
