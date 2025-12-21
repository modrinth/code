import type { Project, Report, Thread, User, Version } from '@modrinth/utils'

export interface OwnershipTarget {
	name: string
	slug: string
	avatar_url?: string
	type: 'user' | 'organization'
}

export interface ExtendedReport extends Report {
	thread: Thread
	reporter_user: User
	project?: Project
	user?: User
	version?: Version
	target?: OwnershipTarget
}
