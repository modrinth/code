export namespace Paper {
	export namespace Versions {
		export namespace v3 {
			export type Project = {
				project: { id: string; name: string }
				versions: Record<string, string[]>
			}

			export type VersionBuilds = {
				builds: number[]
			}
		}
	}
}
