export namespace Archon {
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
		}

		export namespace v1 {
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
		export namespace v1 {
			export type BackupState = 'ongoing' | 'done' | 'failed' | 'cancelled' | 'unchanged'
			export type BackupTask = 'file' | 'create' | 'restore'

			export type BackupTaskProgress = {
				progress: number // 0.0 to 1.0
				state: BackupState
			}

			export type Backup = {
				id: string
				name: string
				created_at: string
				locked: boolean
				automated: boolean
				interrupted: boolean
				ongoing: boolean
				task?: {
					file?: BackupTaskProgress
					create?: BackupTaskProgress
					restore?: BackupTaskProgress
				}
				// TODO: Uncomment when API supports these fields
				// size?: number // bytes
				// creator_id?: string // user ID, or 'auto' for automated backups
			}

			export type BackupRequest = {
				name: string
			}

			export type PatchBackup = {
				name?: string
			}

			export type PostBackupResponse = {
				id: string
			}
		}
	}

	export namespace Websocket {
		export namespace v0 {
			export type WSAuth = {
				url: string
				token: string
			}

			export type BackupState = 'ongoing' | 'done' | 'failed' | 'cancelled' | 'unchanged'
			export type BackupTask = 'file' | 'create' | 'restore'

			export type WSBackupProgressEvent = {
				event: 'backup-progress'
				id: string
				task: BackupTask
				state: BackupState
				progress: number
			}

			export type WSLogEvent = {
				event: 'log'
				stream: 'stdout' | 'stderr'
				message: string
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

			export type WSFilesystemOpsEvent = {
				event: 'filesystem-ops'
				all: FilesystemOperation[]
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
				| WSStatsEvent
				| WSPowerStateEvent
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
