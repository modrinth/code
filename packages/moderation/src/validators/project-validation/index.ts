import type { Labrinth } from '@modrinth/api-client'

import {
	type ProjectTextValidationResult,
	validateProjectDescription,
	validateProjectSummary,
	validateProjectText,
	validateProjectTitle,
} from '../project-fields/index.ts'

export type ProjectValidationField =
	| 'name'
	| 'summary'
	| 'description'
	| 'gallery-name'
	| 'gallery-description'

export interface ProjectValidationFailure extends ProjectTextValidationResult {
	field: ProjectValidationField
	galleryIndex?: number
	galleryUrl?: string
}

export interface ProjectValidationResult {
	valid: boolean
	failures: ProjectValidationFailure[]
}

export function validateProjectFields(
	project: Labrinth.Projects.v3.Project,
): ProjectValidationResult {
	const failures: ProjectValidationFailure[] = []

	function addFailures(
		field: ProjectValidationField,
		results: ProjectTextValidationResult[],
		details: Pick<ProjectValidationFailure, 'galleryIndex' | 'galleryUrl'> = {},
	) {
		failures.push(
			...results.map((result) => ({
				...result,
				field,
				...details,
			})),
		)
	}

	addFailures('name', validateProjectTitle(project.name))
	addFailures('summary', validateProjectSummary(project.summary, project.name))
	addFailures('description', validateProjectDescription(project.description))

	project.gallery.forEach((item, galleryIndex) => {
		const details = {
			galleryIndex,
			galleryUrl: item.url,
		}

		addFailures('gallery-name', validateProjectText(item.name), details)
		addFailures('gallery-description', validateProjectText(item.description), details)
	})

	return {
		valid: !failures.some((failure) => failure.severity === 'error'),
		failures,
	}
}

export function hasProjectFieldValidationFailures(project: Labrinth.Projects.v3.Project): boolean {
	return !validateProjectFields(project).valid
}
