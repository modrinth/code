export const FAVORITES_GROUP_ID = 'group:favorites'
export const MAX_INSTANCE_GROUP_NAME_LENGTH = 256

export type InstanceGroupDefinition = {
	id: string
	name: string
}

export type InstanceGroupMembershipUpdate = {
	instance_id: string
	group_ids: string[]
}
