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

type ProjectEdit = { patch: Labrinth.Projects.v3.EditProjectRequest } | { icon: File | null }

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
		mutationFn: ({ id, edit }: { id: string; edit: ProjectEdit }) => {
			if ('patch' in edit) return client.labrinth.projects_v3.edit(id, edit.patch)
			if (!edit.icon) return client.labrinth.projects_v3.deleteIcon(id)
			return client.labrinth.projects_v3.changeIcon(
				id,
				edit.icon,
				edit.icon.name.split('.').pop() ?? '',
			)
		},
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

	async function save(edit: ProjectEdit) {
		const id = draftProjectId
		const saveSession = session
		if (!id || id !== project.value?.id || mutation.isPending.value) return false
		try {
			await mutation.mutateAsync({ id, edit })
			return session === saveSession && project.value?.id === id
		} catch {
			return false
		}
	}

	return {
		project,
		saving: mutation.isPending,
		beginEditing,
		saveProject: (patch: Labrinth.Projects.v3.EditProjectRequest) => save({ patch }),
		saveIcon: (icon: File | null) => save({ icon }),
	}
}
