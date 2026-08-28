import { getNags } from '../data/nags.ts'
import type { Nag, ProjectValidationContext } from '../types/nags.ts'

export interface ProjectValidationResult {
	valid: boolean
	requiredNags: Nag[]
	warningNags: Nag[]
}

export function validateProject(context: ProjectValidationContext): ProjectValidationResult {
	const nags = getNags(context)
	const requiredNags = nags.filter((nag) => nag.status === 'required')
	const warningNags = nags.filter((nag) => nag.status === 'warning')

	return {
		valid: requiredNags.length === 0,
		requiredNags,
		warningNags,
	}
}
