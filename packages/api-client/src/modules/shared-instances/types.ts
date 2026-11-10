import type { IconConfig } from '../../types/icons'

export namespace SharedInstances {
	export namespace Invites {
		export namespace v1 {
			export type Inviter = {
				id: string
				name: string
				avatar: string | null
			}

			export type UserManager = {
				id: string
				name: string
				type: 'user'
				avatar?: string | null
			}

			export type ServerManager = {
				name: string
				type: 'server'
				icon?: string | null
				icon_config?: IconConfig | null
			}

			export type Manager = UserManager | ServerManager

			export type InviteUser = {
				id: string
				name: string
				avatar?: string | null
				joined_at: string | null
			}

			export type InviteLink = {
				id: string
				expiration: string
				max_uses: number
				uses: number
			}

			export type Invite = {
				instance_id: string
				instance_name: string
				instance_icon?: string | null
				instance_icon_config?: IconConfig | null
				game_version: string
				loader_version: string
				inviter: Inviter | null
				managers: Manager[]
				instance_users?: InviteUser[]
				version?: Instances.v1.InstanceVersion
			}
		}
	}

	export namespace Instances {
		export namespace v1 {
			export type OnlineStatus = 'unknown' | 'running' | 'stopped' | null

			export type Instance = {
				name: string
				icon: string | null
				icon_config: IconConfig | null
				quarantine: boolean
				quarantine_details: QuarantineDetails | null
				installed_once: boolean
				linked_server: { domain: string; region: string; online_status: OnlineStatus } | null
			}

			export type QuarantineScope = 'instance' | 'user'

			export type QuarantineDetails = {
				scope: QuarantineScope
				/** Only returned to the instance's managers and moderators. */
				reason: string | null
				/** Only set when `scope` is `'instance'`. */
				quarantined_at: string | null
				/** Moderator user ID, only returned to moderators. */
				quarantined_by: string | null
				/** Whether the owner was notified, only returned to moderators. */
				owner_notified: boolean | null
			}

			export type InstanceLimits = {
				current: number
				max: number
			}

			export type JoinType = 'owner' | 'invite' | 'link'

			export type InstanceUser = {
				id: string
				joined_at: string | null
				join_type: JoinType
				last_played: string | null
			}

			export type InstanceUsers = {
				users: InstanceUser[]
			}

			export type FileMetadata = {
				path: string
				hash: string
			}

			export type ExternalFile = {
				file_name: string
				file_type: string
				url: string | null
				file_size?: number
				/** SHA-512 hash of the stored file, once it has been uploaded. */
				sha512?: string
				metadata?: FileMetadata[]
			}

			export type InstanceVersion = {
				version: number
				modrinth_ids?: string[]
				ready: boolean
				external_files: ExternalFile[]
				modpack_id: string | null
				removed_files: (
					| { type: 'version'; version_id: string }
					| { type: 'path'; parent: string; filename: string }
				)[]
				game_version: string
				loader: string
				loader_version: string
			}
		}
	}

	export namespace Moderation {
		export namespace v1 {
			export type BlacklistUserRequest = {
				user_ids: string[]
			}

			export type QuarantineInstanceRequest = {
				/** Up to 2000 characters. */
				reason?: string
				/** Whether to notify the instance owner. Defaults to `true`. */
				notify?: boolean
			}
		}
	}

	export namespace Users {
		export namespace v1 {
			export type BlacklistStatus = {
				blacklisted: boolean
				/** Null for users banned before this was recorded. */
				blacklisted_at: string | null
				/** Moderator user ID, only returned to moderators. Null for users banned before this was recorded. */
				blacklisted_by: string | null
			}
		}
	}
}
