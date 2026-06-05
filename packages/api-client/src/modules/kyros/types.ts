export namespace Kyros {
	export namespace UploadSessions {
		export namespace v1 {
			export type Scope = 'content' | 'files'
			export type UploadSessionStatus =
				| 'active'
				| 'uploading'
				| 'finalizing'
				| 'cancelled'
				| 'finalized'
				| 'expired'

			export interface UploadSessionResponse {
				upload_id: string
				status: UploadSessionStatus
				created_at: number
				updated_at: number
				last_upload_at: number | null
				expires_at: number
				entry_count: number
				uploaded_byte_count: number
			}

			export interface GetUploadSessionResponse {
				session: UploadSessionResponse | null
			}
		}
	}

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
