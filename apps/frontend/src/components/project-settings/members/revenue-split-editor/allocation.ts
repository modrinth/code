export const fullAllocation = 10_000

export interface MemberWeight {
	id: string
	name: string
	color: string
	percentage: number
	shareEdited?: boolean
	fixed?: boolean
	organizationMember?: boolean
}

export function toAllocationUnits(percentage: number) {
	const scaled = percentage * 100
	return Math.round(scaled + Number.EPSILON * Math.abs(scaled))
}
