export namespace Kyros {
	export namespace Files {
		export namespace v0 {
			export interface DirectoryItem {
				name: string
				type: 'file' | 'directory' | 'symlink'
				path: string
				modified: number
				created: number
				size?: number
				count?: number
				target?: string
			}

			export interface DirectoryResponse {
				items: DirectoryItem[]
				total: number
				current: number
			}

			export interface ExtractResult {
				modpack_name: string | null
				conflicting_files: string[]
			}
		}
	}
}
