import type { Labrinth } from '@modrinth/api-client'
import {
	commonMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { watch } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

export function useProjectInfoEdit() {
	const { project } = injectProjectReviewPageContext()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { addNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()
	let draftProjectId: string | undefined
	let session = 0

	function beginEditing() {
		draftProjectId = project.value?.id
		session += 1
	}

	watch(
		() => project.value?.id,
		() => {
			draftProjectId = undefined
			session += 1
		},
		{ flush: 'sync' },
	)

	const mutation = useMutation({
		mutationFn: ({ id, patch }: { id: string; patch: Labrinth.Projects.v3.EditProjectRequest }) =>
			client.labrinth.projects_v3.edit(id, patch),
		onSuccess: async (_, { id }) => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['project', 'v3', id] }),
				queryClient.invalidateQueries({ queryKey: ['project', 'v2', id] }),
				queryClient.invalidateQueries({ queryKey: ['project', id] }),
			])
		},
		onError: (error) => {
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: error instanceof Error ? error.message : String(error),
				type: 'error',
			})
		},
	})

	async function saveProject(patch: Labrinth.Projects.v3.EditProjectRequest) {
		const id = draftProjectId
		const saveSession = session
		if (!id || id !== project.value?.id || mutation.isPending.value) return false
		try {
			await mutation.mutateAsync({ id, patch })
			return session === saveSession && project.value?.id === id
		} catch {
			return false
		}
	}

	return { project, saving: mutation.isPending, beginEditing, saveProject }
}
