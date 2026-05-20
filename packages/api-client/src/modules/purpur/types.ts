export namespace Purpur {
	export namespace Versions {
		export namespace v2 {
			export type Project = {
				project: string
				versions: string[]
			}

			export type VersionBuilds = {
				builds: {
					all: string[]
				}
			}
		}
	}
}
