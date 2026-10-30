export const IssuePriority = {
	Top: -1000,
	Alerts: -40,
	Rules: -30,
	Rejected: -20,
	Withheld: -10,
	Default: 0,
	Note: 10,
	Bottom: 20,
} as const

export type IssuePriority = (typeof IssuePriority)[keyof typeof IssuePriority]
