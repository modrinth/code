export type ModerationStatus = 'approved' | 'rejected' | 'flagged'

export type ReviewStateValue =
	| boolean
	| string
	| number
	| Set<string>
	| { [key: string]: ReviewStateValue }
	| null
	| undefined
