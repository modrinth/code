import type { Labrinth } from '../labrinth/types'

export namespace Archon {
	export namespace Content {
		export namespace v1 {
			export type AddonKind = 'mod' | 'plugin' | 'datapack' | 'shader' | 'resourcepack'

			export type ContentOwnerType = 'user' | 'organization'

			export type ContentOwner = {
				id: string
				name: string
				type: ContentOwnerType
				icon_url: string | null
			}

			export type AddonVersion = {
				id: string
				name: string | null
				environment?: Labrinth.Projects.v3.Environment | null
			}

			export type Addon = {
				id: string
				filename: string
				filesize: number
				disabled: boolean
				kind: AddonKind
				from_modpack: boolean
				pack_client_retained: boolean
				pack_client_depends: boolean
				has_update: string | null
				name: string | null
				project_id: string | null
				version: AddonVersion | null
				owner: ContentOwner | null
				icon_url: string | null
			}

			export type Addons = {
				modloader: string | null
				modloader_version: string | null
				game_version: string | null
				modpack: ModpackFields | null
				addons: Addon[] | null
			}

			export type AddAddonRequest = {
				project_id: string
				version_id?: string
				kind?: AddonKind
			}

			export type AddAddonsRequest = AddAddonRequest[]

			export type RemoveAddonRequest = {
				kind: AddonKind
				filename: string
			}

			export type UpdateAddonRequest = {
				filename: string
				version_id?: string | null
			}

			export type Modloader =
				| 'forge'
				| 'neo_forge'
				| 'fabric'
				| 'quilt'
				| 'paper'
				| 'purpur'
				| 'vanilla'

			export type ModpackSpecModrinth = {
				platform: 'modrinth'
				project_id: string
				version_id: string
			}

			export type ModpackSpecLocalFile = {
				platform: 'local_file'
				filename: string
				name: string
				description: string | null
			}

			export type ModpackSpec = ModpackSpecModrinth | ModpackSpecLocalFile

			export type ModpackOwner = {
				id: string
				name: string
				type: 'user' | 'organization'
				icon_url: string | null
			}

			export type ModpackFields = {
				spec: ModpackSpec
				has_update: string | null
				title: string | null
				description: string | null
				icon_url: string | null
				owner: ModpackOwner | null
				version_number: string | null
				date_published: string | null
				downloads: number | null
				followers: number | null
			}

			export type KnownPropertiesFields = {
				allow_cheats?: string | null
				allow_flight?: string | null
				difficulty?: string | null
				enforce_whitelist?: string | null
				force_gamemode?: string | null
				gamemode?: string | null
				generate_structures?: string | null
				generator_settings?: string | null
				hardcore?: string | null
				level_seed?: string | null
				level_type?: string | null
				max_players?: string | null
				max_tick_time?: string | null
				motd?: string | null
				pause_when_empty_seconds?: string | null
				player_idle_timeout?: string | null
				require_resource_pack?: string | null
				resource_pack?: string | null
				resource_pack_id?: string | null
				resource_pack_sha1?: string | null
				simulation_distance?: string | null
				spawn_protection?: string | null
				sync_chunk_writes?: string | null
				view_distance?: string | null
				white_list?: string | null
			}

			export type PropertiesFields = {
				known: KnownPropertiesFields
				custom?: Record<string, string>
			}

			export type PatchPropertiesFields = {
				known?: KnownPropertiesFields
				custom?: Record<string, string | null>
			}

			export type JreVendor = 'temurin' | 'corretto' | 'graal'

			export type RuntimeOptions = {
				java_version: number | null
				jre_vendor: JreVendor | null
				original_invocation: string | null
				startup_command: string | null
			}

			export type PatchRuntimeOptions = {
				java_version?: number | null
				jre_vendor?: JreVendor | null
				startup_command?: string | null
			}

			export type InstallWorldContent =
				| {
						content_variant: 'modpack'
						spec: ModpackSpec
						soft_override: boolean
						properties?: PropertiesFields | null
				  }
				| {
						content_variant: 'bare'
						loader: Modloader
						version: string
						game_version?: string
						soft_override: boolean
						properties?: PropertiesFields | null
				  }

			export type AddonDiffVersion = {
				id: string
				version_number: string
			}

			export type AddonDiffProject = {
				id: string
				title: string
				icon_url: string | null
				slug: string
			}

			export type AddonBaseDiffInfo = {
				current_version: AddonDiffVersion | null
				new_version: AddonDiffVersion | null
				file_name: string | null
				project_id: string | null
				project: AddonDiffProject | null
			}

			export type AddonDiffAdded = AddonBaseDiffInfo & {
				type: 'added'
				new_version_id: string
			}

			export type AddonDiffRemoved = AddonBaseDiffInfo & {
				type: 'removed'
			}

			export type AddonDiffUpdated = AddonBaseDiffInfo & {
				type: 'updated'
				current_version_id: string
				new_version_id: string
			}

			export type AddonDiff = AddonDiffAdded | AddonDiffRemoved | AddonDiffUpdated

			export type UpdateGameVersionPreview = {
				addon_changes: AddonDiff[]
				new_game_version: string
				new_loader_version: string
				has_unknown_content: boolean
			}
		}
	}

	export namespace Servers {
		export namespace v0 {
			export type ServerGetResponse = {
				servers: Server[]
				pagination: Pagination
			}

			export type Pagination = {
				current_page: number
				page_size: number
				total_pages: number
				total_items: number
			}

			export type Status = 'installing' | 'broken' | 'available' | 'suspended'

			export type SuspensionReason =
				| 'moderated'
				| 'paymentfailed'
				| 'cancelled'
				| 'upgrading'
				| 'other'

			export type Loader =
				| 'Forge'
				| 'NeoForge'
				| 'Fabric'
				| 'Quilt'
				| 'Purpur'
				| 'Spigot'
				| 'Vanilla'
				| 'Paper'

			export type Game = 'Minecraft'

			export type UpstreamKind = 'modpack' | 'none'

			export type Server = {
				server_id: string
				name: string
				owner_id: string
				net: Net
				game: Game
				backup_quota: number
				used_backup_quota: number
				status: Status
				suspension_reason: SuspensionReason | null
				loader: Loader | null
				loader_version: string | null
				mc_version: string | null
				upstream: Upstream | null
				sftp_username: string
				sftp_password: string
				sftp_host: string
				datacenter: string
				notices: Notice[]
				node: NodeInfo | null
				flows: Flows
				is_medal: boolean

				medal_expires?: string
			}

			export type Net = {
				ip: string
				port: number
				domain: string
			}

			export type Upstream = {
				kind: UpstreamKind
				version_id: string
				project_id: string
			}

			export type Notice = {
				id: number
				dismissable: boolean
				title: string
				message: string
				level: string
				announced: string
			}

			export type NodeInfo = {
				token: string
				instance: string
			}

			export type Flows = {
				intro: boolean
			}

			export type GetServersOptions = {
				limit?: number
				offset?: number
			}

			export type StockRequest = {
				cpu?: number
				memory_mb?: number
				swap_mb?: number
				storage_mb?: number
			}

			export type StockResponse = {
				available: number
			}

			export type JWTAuth = {
				url: string // e.g., "node-xyz.modrinth.com/modrinth/v0/fs"
				token: string // JWT token for filesystem access
			}

			export type ReinstallLoaderRequest = {
				loader: string
				loader_version?: string
				game_version?: string
			}

			export type ReinstallModpackRequest = {
				project_id: string
				version_id?: string
			}

			export type ReinstallRequest = ReinstallLoaderRequest | ReinstallModpackRequest

			export type MrpackReinstallAuth = {
				url: string
				token: string
			}

			export type Allocation = {
				port: number
				name: string
			}

			export type StartupConfig = {
				invocation: string
				original_invocation: string
				jdk_version: 'lts8' | 'lts11' | 'lts17' | 'lts21'
				jdk_build: 'corretto' | 'temurin' | 'graal'
			}
		}

		export namespace v1 {
			export type ServerFull = {
				id: string
				name: string
				subdomain: string
				specs: ServerResources
				sftp_username: string
				sftp_password: string
				tags: string[]
				location: ServerLocation
				worlds: WorldFull[]
			}

			export type ServerResources = {
				cpu: number
				memory_mb: number
				storage_mb: number
				swap_mb: number
			}

			export type ServerLocation =
				| {
						status: 'assigned'
						location_metadata: {
							region: string
							region_should_be_user_displayed: boolean
							hostname: string
							is_decommissioned_node: boolean
						}
				  }
				| {
						status: 'unassigned'
				  }

			export type WorldFull = {
				id: string
				name: string
				created_at: string
				is_active: boolean
				/**
				 * @deprecated Prefer `client.archon.backups_queue_v1.list()` for queue-aware backup state.
				 */
				backups: Archon.Backups.v1.Backup[]
				content: WorldContentInfo | null
				readiness: WorldReadiness
			}

			export type WorldReadiness = {
				data_synchronized_fetched: boolean
			}

			export type WorldContentInfo = {
				modloader: string
				modloader_version: string
				game_version: string
				java_version: number
				invocation: string
				original_invocation: string
			}

			export type Region = {
				shortcode: string
				country_code: string
				display_name: string
				lat: number
				lon: number
				zone: string
			}
		}
	}

	export namespace Backups {
		/**
		 * @deprecated Use {@link Archon.BackupsQueue.v1} and `client.archon.backups_queue_v1` instead.
		 */
		export namespace v1 {
			/** @deprecated Use {@link Archon.BackupsQueue.v1} instead. */
			export type BackupState = 'ongoing' | 'done' | 'failed' | 'cancelled' | 'unchanged'
			/** @deprecated Use {@link Archon.BackupsQueue.v1} instead. */
			export type BackupTask = 'file' | 'create' | 'restore'
			/** @deprecated Use {@link Archon.BackupsQueue.v1} instead. */
			export type BackupStatus = 'pending' | 'in_progress' | 'timed_out' | 'error' | 'done'

			/** @deprecated Use {@link Archon.BackupsQueue.v1} instead. */
			export type BackupTaskProgress = {
				progress: number // 0.0 to 1.0
				state: BackupState
			}

			/** @deprecated Use {@link Archon.BackupsQueue.v1.BackupQueueBackup} instead. */
			export type Backup = {
				id: string
				physical_id: string
				name: string
				created_at: string
				automated: boolean
				status: BackupStatus
				interrupted: boolean
				ongoing: boolean
				locked: boolean
				task?: {
					file?: BackupTaskProgress
					create?: BackupTaskProgress
					restore?: BackupTaskProgress
				}
			}

			/** @deprecated Use {@link Archon.BackupsQueue.v1.BackupRequest} instead. */
			export type BackupRequest = {
				name: string
			}

			/** @deprecated Use {@link Archon.BackupsQueue.v1} instead. */
			export type PatchBackup = {
				name?: string
			}

			/** @deprecated Use {@link Archon.BackupsQueue.v1.PostBackupQueueResponse} instead. */
			export type PostBackupResponse = {
				id: string
			}
		}
	}

	export namespace BackupsQueue {
		export namespace v1 {
			export type BackupQueueOperationType = 'create' | 'restore'

			export type BackupQueueState =
				| 'pending'
				| 'ongoing'
				| 'completed'
				| 'cancelled'
				| 'failed'
				| 'timed_out'

			export type BackupStatus = 'pending' | 'in_progress' | 'timed_out' | 'error' | 'done'

			export type BackupRequest = {
				name: string
			}

			export type PostBackupQueueResponse = {
				id: string
			}

			export type DeleteManyBackupRequest = {
				backup_ids: string[]
			}

			export type ActiveOperation = {
				backup_id: string
				operation_type: BackupQueueOperationType
				operation_id?: number | null
				has_parent: boolean
				scheduled_for: string
				synthetic_legacy: boolean
			}

			export type BackupQueueOperation = {
				operation_type: BackupQueueOperationType
				operation_id?: number | null
				state: BackupQueueState
				scheduled_for: string
				completed_at?: string | null
				has_parent: boolean
				error?: string | null
				should_prompt: boolean
				synthetic_legacy: boolean
			}

			export type BackupQueueBackup = {
				id: string
				name: string
				created_at: string
				status: BackupStatus
				locked: boolean
				automated: boolean
				history: BackupQueueOperation[]
			}

			export type BackupsQueueResponse = {
				active_operations: ActiveOperation[]
				backups: BackupQueueBackup[]
			}
		}
	}

	export namespace Websocket {
		export namespace v0 {
			export type WSAuth = {
				url: string
				token: string
			}

			export type BackupState =
				| 'pending'
				| 'ongoing'
				| 'done'
				| 'failed'
				| 'cancelled'
				| 'unchanged'
				| 'damaged'
			export type BackupTask = 'file' | 'create' | 'restore'

			export type WSBackupProgressEvent = {
				event: 'backup-progress'
				id: string
				task: BackupTask
				state: BackupState
				progress: number
				start_time?: number | null
				finish_time?: number | null
			}

			export type WSLogEvent = {
				event: 'log'
				stream: 'stdout' | 'stderr'
				message: string
			}

			export type WSLog4jEvent = {
				event: 'log4j'
				logger_name?: string
				level?: string
				thread_name?: string
				timestamp_millis?: number
				message?: string
				throwable?: string
			}

			export type WSStatsEvent = {
				event: 'stats'
				cpu_percent: number
				ram_usage_bytes: number
				ram_total_bytes: number
				storage_usage_bytes: number
				storage_total_bytes: number
				net_tx_bytes: number
				net_rx_bytes: number
			}

			export type PowerState = 'running' | 'stopped' | 'starting' | 'stopping' | 'crashed'

			export type WSPowerStateEvent = {
				event: 'power-state'
				state: PowerState
				oom_killed?: boolean
				exit_code?: number
			}

			export type WSAuthExpiringEvent = {
				event: 'auth-expiring'
			}

			export type WSAuthIncorrectEvent = {
				event: 'auth-incorrect'
			}

			export type WSAuthOkEvent = {
				event: 'auth-ok'
			}

			export type WSInstallationResultEvent =
				| WSInstallationResultOkEvent
				| WSInstallationResultErrEvent

			export type WSInstallationResultOkEvent = {
				event: 'installation-result'
				result: 'ok'
			}

			export type WSInstallationResultErrEvent = {
				event: 'installation-result'
				result: 'err'
				reason?: string
			}

			export type WSUptimeEvent = {
				event: 'uptime'
				uptime: number
			}

			export type WSNewModEvent = {
				event: 'new-mod'
				project_id: string
				version_id: string
			}

			export type FilesystemOpKind = 'unarchive'

			export type FilesystemOpState =
				| 'queued'
				| 'ongoing'
				| 'done'
				| 'cancelled'
				| 'failure-corrupted'
				| 'failure-invalid-path'

			export type FilesystemOperation = {
				op: FilesystemOpKind
				id: string
				progress: number
				bytes_processed: number
				files_processed: number
				state: FilesystemOpState
				mime: string
				current_file?: string
				invalid_path?: string
				src: string
				started: string
			}

			export type QueuedFilesystemOp = {
				op: FilesystemOpKind
				src: string
			}

			export type WSFilesystemOpsEvent = {
				event: 'filesystem-ops'
				all: FilesystemOperation[]
			}

			export type ReadinessState =
				| 'deprovisioned'
				| 'waiting_active_world'
				| 'waiting_world_spec_details_for_progress'
				| 'pulling_world_data'
				| 'migration_zfs'
				| 'sync_content'
				| 'container_readying'
				| 'ready'

			export type FlattenedPowerState = 'not_ready' | 'starting' | 'running' | 'stopping' | 'idle'

			export type SyncInstallPhase = 'Analyzing' | 'InstallingPack' | 'InstallingLoader' | 'Addons'

			export type SyncContentProgress = {
				started_at: string
				phase: SyncInstallPhase
				percent: number
			}

			export type SyncContentError = {
				step: string
				description: string
			}

			export type WSStateEvent = {
				event: 'state'
				debug: string
				power_variant: FlattenedPowerState
				exit_code?: number | null
				was_oom?: boolean
				target: 'start' | 'stop' | 'restart' | null
				uptime: number
				progress: SyncContentProgress | null
				content_error: SyncContentError | null
			}

			// Outgoing messages (client -> server)
			export type WSOutgoingMessage = WSAuthMessage | WSCommandMessage

			export type WSAuthMessage = {
				event: 'auth'
				jwt: string
			}

			export type WSCommandMessage = {
				event: 'command'
				cmd: string
			}

			export type WSEvent =
				| WSBackupProgressEvent
				| WSLogEvent
				| WSLog4jEvent
				| WSStatsEvent
				| WSPowerStateEvent
				| WSStateEvent
				| WSAuthExpiringEvent
				| WSAuthIncorrectEvent
				| WSAuthOkEvent
				| WSInstallationResultEvent
				| WSUptimeEvent
				| WSNewModEvent
				| WSFilesystemOpsEvent

			export type WSEventType = WSEvent['event']
		}
	}
}
