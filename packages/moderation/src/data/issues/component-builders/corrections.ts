import type { IssueCorrections } from './types'

export interface CorrectionConflict {
	target: 'project' | 'version'
	versionId?: string
	field: string
	issueIds: string[]
}

/** Combines selected issues' patches without silently overwriting conflicting corrections. */
export function aggregateCorrections(
	issues: readonly { id: string; corrections?: IssueCorrections }[],
) {
	const project: NonNullable<IssueCorrections['project']> = {}
	const versions: NonNullable<IssueCorrections['versions']> = {}
	const conflicts: CorrectionConflict[] = []
	const fields = new Map<
		string,
		{ value: unknown; issueIds: string[]; conflict?: CorrectionConflict }
	>()

	function merge(
		patch: object,
		output: object,
		issueId: string,
		target: 'project' | 'version',
		versionId?: string,
	) {
		for (const [field, value] of Object.entries(patch)) {
			if (value === undefined) continue
			const key = JSON.stringify([target, versionId, field])
			const previous = fields.get(key)
			if (!previous) {
				fields.set(key, { value, issueIds: [issueId] })
				Object.assign(output, { [field]: value })
				continue
			}
			previous.issueIds.push(issueId)
			if (previous.conflict) continue
			if (JSON.stringify(previous.value) === JSON.stringify(value)) continue
			const conflict = { target, versionId, field, issueIds: previous.issueIds }
			previous.conflict = conflict
			conflicts.push(conflict)
			Reflect.deleteProperty(output, field)
		}
	}

	for (const { id, corrections } of issues) {
		if (corrections?.project) merge(corrections.project, project, id, 'project')
		for (const [versionId, patch] of Object.entries(corrections?.versions ?? {})) {
			const output = versions[versionId] ?? {}
			merge(patch, output, id, 'version', versionId)
			versions[versionId] = output
		}
	}

	return {
		project,
		versions,
		conflicts,
		issueIds: issues
			.filter(
				({ corrections }) =>
					corrections &&
					(Object.values(corrections.project ?? {}).some((value) => value !== undefined) ||
						Object.values(corrections.versions ?? {}).some((patch) =>
							Object.values(patch).some((value) => value !== undefined),
						)),
			)
			.map(({ id }) => id),
	}
}
