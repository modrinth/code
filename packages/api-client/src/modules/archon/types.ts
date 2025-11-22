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

	export namespace Websocket {
		export namespace v0 {
			export type WSAuth = {
				url: string
				token: string
			}

			export type BackupState = 'ongoing' | 'done' | 'failed' | 'cancelled' | 'unchanged'
			export type BackupTask = 'create' | 'restore'

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
				cpu_used: number
				memory_used: number
				memory_total: number
				swap_used: number
				swap_total: number
				disk_used: number
				disk_total: number
				uptime: number
			}

			export type PowerState = 'running' | 'stopped' | 'starting' | 'stopping' | 'crashed'

			export type WSPowerStateEvent = {
				event: 'power-state'
				state: PowerState
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

			export type WSInstallationResultEvent = {
				event: 'installation-result'
				success: boolean
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

			export type FilesystemOp = 'unarchive'

			export type WSFilesystemOpsEvent = {
				event: 'filesystem-ops'
				op: FilesystemOp
				progress: number
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
