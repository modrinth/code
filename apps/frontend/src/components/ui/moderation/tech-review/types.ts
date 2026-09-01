import type { Labrinth } from '@modrinth/api-client'

export type FlattenedFileReport = Labrinth.TechReview.Internal.FileReport & {
	id: string
	version_id: string
	version_number?: string
}

export type TechRevProjectRef = {
	id: string
	slug?: string
	project_types: string[]
}

export type DetailDecision = 'safe' | 'malware' | 'pending'
export type DetailDecisionScope = 'local' | 'global'

export type FlagItem = {
	issueId: string
	issueType: string
	detail: Labrinth.TechReview.Internal.ReportIssueDetail
}

export type ClassGroup = {
	key: string
	jar: string | null
	filePath: string
	flags: FlagItem[]
}

export type JarGroup = {
	key: string
	jar: string | null
	segments: string[]
	classes: ClassGroup[]
}

export type TraceVerdictEvent = {
	detail: Labrinth.TechReview.Internal.ReportIssueDetail
	decision: Exclude<DetailDecision, 'pending'>
	scope: DetailDecisionScope
}
