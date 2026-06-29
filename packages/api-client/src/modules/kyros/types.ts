export namespace Kyros {
	export namespace UploadSessions {
		export namespace v1 {
			export type Scope = 'content' | 'files'

			export type UploadSessionFile = {
				file: File | Blob
				filename: string
			}

			export interface UploadSessionResponse {
				upload_id: string
				status: string
				created_at: number
				updated_at: number
				last_upload_at?: number | null
				expires_at: number
				entry_count: number
				uploaded_byte_count: number
			}

			export interface GetUploadSessionResponse {
				session?: UploadSessionResponse | null
			}
		}
	}

	export namespace Files {
		export namespace v1 {
			export type DescendantType = 'regular' | 'directory' | 'symlink' | 'other'

			export type UnzipSource =
				| {
						type: 'zip_url'
						url: string
				  }
				| {
						type: 'zip_path'
						path: string
				  }

			export interface CreateDownloadSessionRequest {
				path: string
				zipped: boolean
			}

			export interface DeleteFileRequest {
				path: string
			}

			export interface FileListingItem {
				name: string
				full_path: string
				size_bytes: number
				type: DescendantType
				mtime: string
				ctime: string
				descendants: number
			}

			export interface FileListingRequest {
				path: string
				page: number
				items_per_page: number
			}

			export interface FileListingResponse {
				items: FileListingItem[]
				page: number
				items_per_page: number
				page_total: number
				items_total: number
				too_many_descendants: boolean
				descendants_limit: number
				digest?: string | null
			}

			export interface FileMutationResponse {
				source: string
				destination: string
			}

			export interface MoveFileRequest {
				source: string
				destination: string
			}

			export interface RenameFileRequest {
				path: string
				name: string
			}

			export interface PathMutationRequest {
				path: string
			}

			export interface UnzipFileRequest {
				source: UnzipSource
				target: string
			}
		}
	}
}
