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
				/** A file of the modpack to select instead of the file selected by the link. */
				file_id?: number | null
			}

			export type Loader = 'forge' | 'neoforge' | 'fabric' | 'quilt'

			export type ReleaseChannel = 'release' | 'beta' | 'alpha'

			export interface LinkQueryFile {
				id: number
				display_name: string
				file_name: string
				file_date: string
				file_length: number
				/** Mixes Minecraft versions, loader names and other tags. */
				game_versions: string[]
				minecraft_versions: string[]
				loaders: Loader[]
			}

			export interface ProjectFile extends LinkQueryFile {
				/** `null` for a channel unknown to Kyros. */
				release_channel: ReleaseChannel | null
				/** Whether the file can be downloaded from CurseForge, and so installed from its link. */
				available: boolean
				/** Whether CurseForge lists a server pack for the file, its availability is unchecked. */
				has_server_pack: boolean
				ftb_installer_hint: boolean
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
				/** Whether the file may be installed with FTB's server installer instead of a server pack. */
				ftb_installer_hint: boolean
				/**
				 * When the file has no server pack, the nearest file for the same Minecraft version and
				 * loader with an available server pack.
				 */
				closest_server_pack_file?: ProjectFile | null
			}

			export type ProjectFilesRequest = {
				game_version?: string
				index?: number
				/** Between 1 and 50, 50 by default. */
				page_size?: number
			}

			export interface ProjectFilesResponse {
				files: ProjectFile[]
				index: number
				page_size: number
				total_count: number
			}

			export interface ExecuteSessionRequest {
				/**
				 * Mutually exclusive with an uploaded client pack. An uploaded server pack replaces the
				 * linked file's server pack.
				 */
				link?: string | null
				/** A file of the linked modpack to install instead of the file selected by the link. */
				file_id?: number | null
				soft_override?: boolean
				/**
				 * Restores the client files of a world installed from a CurseForge modpack, by updating
				 * it to the modpack after backing it up. Implies `soft_override`.
				 */
				restore_client_files?: boolean
				/** With `restore_client_files`, installs right away without backing up the world first. */
				no_backup?: boolean
				properties: Archon.Content.v1.PropertiesFields
			}

			export interface ExecuteSessionResponse {
				/** The backup made before restoring client files, which reports the restoration's progress. */
				backup_id: string | null
			}

			export interface DetectionResponse {
				likely_curseforge: boolean
				non_modrinth_mods: number
				server_pack_creator_variables: boolean
				/**
				 * Whether the world was installed from an uploaded mrpack that is still in its root folder,
				 * so its client files can be restored with `use_existing_mrpack`.
				 */
				has_manually_uploaded_mrpack: boolean
			}
		}
	}

	export namespace Content {
		export namespace v1 {
			export interface UploadModpackFileResponse {
				/** The backup made before restoring client files, which reports the restoration's progress. */
				backup_id: string | null
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

	export namespace Backups {
		export namespace v1 {
			export type BackupDumpFormat = 'zip'

			export interface BackupDownloadAuthorization {
				token: string
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
