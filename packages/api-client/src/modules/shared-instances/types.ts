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

			export type Invite = {
				instance_id: number
				instance_name: string
				game_version: string
				loader_version: string
				managers: Manager[]
			}
		}
	}
}
