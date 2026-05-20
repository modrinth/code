export namespace Paper {
	export namespace Versions {
		export namespace v3 {
			export type Project = {
				project: { id: string; name: string }
				versions: Record<string, string[]>
			}

			export type BuildChannel = 'STABLE' | 'BETA' | 'ALPHA'

			export type Build = {
				id: number
				time: string
				channel: BuildChannel | string
			}

			export type VersionBuilds = {
				builds: Build[]
			}
		}
	}
}
