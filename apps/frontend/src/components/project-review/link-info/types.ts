import type { AbstractModrinthClient } from '@modrinth/api-client'

import type { messages } from './messages'

export type Label = keyof typeof messages
export type Detail = {
	label: Label
	value: string | number | boolean | { message: Label }
	format?: 'date' | 'bytes'
	warning?: Label
}
export type Profile = {
	title: string
	label?: Label
	url?: string
	avatar?: string | null
	description?: string | null
	details: Detail[]
}
export type LinkInfo = {
	profiles: Profile[]
	warnings: Label[]
	links?: { url: string; title: string }[]
}
export type Request = <T>(url: string) => Promise<T>
export type LinkMatch = {
	platform: string
	kind: Label
	name?: string
	load?: (request: Request, client: AbstractModrinthClient) => Promise<LinkInfo>
}
export type Provider = (url: URL) => LinkMatch | undefined

export function detail(
	label: Label,
	value: Detail['value'] | null | undefined,
	format?: Detail['format'],
	warning?: Label,
): Detail[] {
	return value == null || value === '' ? [] : [{ label, value, format, warning }]
}
