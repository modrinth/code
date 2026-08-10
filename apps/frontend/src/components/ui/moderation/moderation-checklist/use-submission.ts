import type { Labrinth } from '@modrinth/api-client'
import type { ActiveAction } from '@modrinth/moderation/src/types/node'
import { createTrackedPatch, hasCap } from '@modrinth/moderation/src/types/node'
import type { FixBuilder } from '@modrinth/moderation/src/types/node/fix'
import { injectModrinthClient } from '@modrinth/ui'
import type { ProjectStatus } from '@modrinth/utils'
import { useMutation } from '@tanstack/vue-query'
import type { Ref } from 'vue'

interface ModerationSubmissionOptions {
	project: Ref<Labrinth.Projects.v3.Project>
	projectV2: Ref<Labrinth.Projects.v2.Project>
	versions: Ref<Labrinth.Versions.v3.Version[] | null>
}

interface ModerationSubmission {
	status: ProjectStatus
	message: string | null
	activeActions: ActiveAction[]
}

function getFixes(node: object): FixBuilder[] {
	return hasCap(node, '_fixes') && Array.isArray(node._fixes)
		? (node._fixes as FixBuilder[])
		: []
}

function shouldApplyFixes(actions: ActiveAction[]): boolean {
	return actions.some(({ node }) => hasCap(node, '_applyFixes') && node._applyFixes === true)
}

export function useModerationSubmission({
	project,
	projectV2,
	versions,
}: ModerationSubmissionOptions) {
	const client = injectModrinthClient()

	return useMutation({
		mutationFn: async ({ status, message, activeActions }: ModerationSubmission) => {
			const projectId = projectV2.value.id
			const threadId = projectV2.value.thread_id

			await client.labrinth.projects_v2.edit(projectId, { status })

			if (message && threadId) {
				await client.labrinth.threads_v3.sendMessage(threadId, {
					body: { type: 'text', body: message },
				})
			}

			let projectFixChanges: Labrinth.Projects.v3.EditProjectRequest = {}
			if (!shouldApplyFixes(activeActions)) return projectFixChanges

			const { proxy: projectProxy, changes: projectChanges } = createTrackedPatch(
				project.value as Labrinth.Projects.v3.EditProjectRequest,
			)
			for (const { node, state } of activeActions) {
				for (const fix of getFixes(node)) fix._projectFn?.(projectProxy, state)
			}
			projectFixChanges = projectChanges()
			if (Object.keys(projectFixChanges).length > 0) {
				await client.labrinth.projects_v3.edit(projectId, projectFixChanges)
			}

			const versionFixes = activeActions.flatMap(({ node, state }) =>
				getFixes(node)
					.filter((fix) => fix._versionFn)
					.map((fix) => ({ fix, state })),
			)
			if (versionFixes.length === 0 || !versions.value) return projectFixChanges

			await Promise.all(
				versions.value.map(async (version) => {
					const { proxy, changes } = createTrackedPatch(
						version as Labrinth.Versions.v3.ModifyVersionRequest,
					)
					for (const { fix, state } of versionFixes) {
						fix._versionFn?.(proxy, state)
					}
					const changed = changes()
					if (Object.keys(changed).length > 0) {
						await client.labrinth.versions_v3.modifyVersion(version.id, changed)
					}
				}),
			)

			return projectFixChanges
		},
	})
}
