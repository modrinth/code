import type { Archon } from '../archon/types'

export namespace Kyros {
	export namespace FlameAnvil {
		export namespace v1 {
			export type SessionStatus = Kyros.UploadSessions.v1.UploadSessionStatus

			export interface UploadedPack {
				filename: string
				size: number
			}

			export interface SessionResponse {
				upload_id: string
				status: SessionStatus
				created_at: number
				updated_at: number
				expires_at: number
				client_pack?: UploadedPack | null
				server_pack?: UploadedPack | null
			}

			export interface GetSessionResponse {
				session: SessionResponse | null
			}

			export interface LinkQueryRequest {
				link: string
			}

			export interface LinkQueryFile {
				id: number
				display_name: string
				file_name: string
				file_date: string
				file_length: number
				game_versions: string[]
			}

			export interface LinkQueryResponse {
				project_id: number
				slug: string
				name: string
				summary?: string | null
				icon_url?: string | null
				authors: string[]
				file: LinkQueryFile
				server_pack?: LinkQueryFile | null
				server_pack_missing: boolean
			}

			export interface ExecuteSessionRequest {
				/** Mutually exclusive with uploaded packs. */
				link?: string | null
				soft_override?: boolean
				properties: Archon.Content.v1.PropertiesFields
			}
		}
	}

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
		export namespace v1 {
			export interface FileDownloadAuthorization {
				token: string
			}

			export interface FullWorldDownloadAuthorization {
				token: string
			}

			export type DescendantType = 'regular' | 'directory' | 'symlink' | 'other'

			export interface FileStatRequest {
				path: string
			}

			export interface FileStatResponse {
				name: string
				full_path: string
				size_bytes: number
				type: DescendantType
				mtime: string
				ctime: string
			}

			export type ZipRequest =
				| {
						target_type: 'Directory'
						path: string
				  }
				| {
						target_type: 'ManyPaths'
						parent: string
						include: string[]
						target: string
				  }

			export interface ZipProgress {
				progress: number
				done?: boolean
				error?: string
			}
		}

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
