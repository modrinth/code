export const MAX_CONTENT_GROUP_NAME_LENGTH = 256

export type InstanceContentGroup = {
	id: string
	content_type: string
	name: string
	item_keys: string[]
}
