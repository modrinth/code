export namespace SharedInstances {
	export namespace Invites {
		export namespace v1 {
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
			}

			export type Manager = UserManager | ServerManager

			export type InviteUser = {
				id: string
				name: string
				avatar?: string | null
			}

			export type Invite = {
				instance_id: string
				instance_name: string
				instance_icon?: string | null
				game_version: string
				loader_version: string
				managers: Manager[]
				instance_users?: InviteUser[]
			}
		}
	}
}
