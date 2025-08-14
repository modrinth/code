import type { DelphiReport, Project, Report, Thread, User, Version } from '@modrinth/utils'

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

export interface ExtendedDelphiReport extends DelphiReport {
	target?: OwnershipTarget
}

export interface ReportQuickReply {
	label: string
	message: string | ((report: ExtendedReport) => Promise<string> | string)
	shouldShow?: (report: ExtendedReport) => boolean
	private?: boolean
}
