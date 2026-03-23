export type ServerNotice = {
	id: number
	message: string
	title?: string
	level: 'info' | 'warn' | 'critical' | 'survey'
	dismissable: boolean
	announce_at: string
	expires: string
	assigned: {
		kind: 'server' | 'node'
		id: string
		name: string
	}[]
	dismissed_by: {
		server: string
		dismissed_on: string
	}[]
}
