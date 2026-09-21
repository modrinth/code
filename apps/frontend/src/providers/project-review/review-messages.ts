import type { Labrinth } from '@modrinth/api-client'
import { expandVariables } from '@modrinth/moderation/src/utils'
import { createContext } from '@modrinth/ui'
import { computed, type Ref, ref, watch } from 'vue'

import type { createReviewPanels } from './review-panels'

export const [injectReviewMessages, provideReviewMessages] =
	createContext<ReturnType<typeof createReviewMessages>>('ProjectReviewMessages')

export function createReviewMessages(
	project: Ref<Labrinth.Projects.v3.Project | undefined>,
	projectV2: Ref<Labrinth.Projects.v2.Project | undefined>,
	panels: ReturnType<typeof createReviewPanels>,
) {
	const draft = ref('')
	const generating = computed(() => !!project.value && projectV2.value?.id !== project.value.id)
	const generated = computed(() => {
		const current = project.value
		const legacy = projectV2.value
		if (!current || !legacy || current.id !== legacy.id) return ''
		return expandVariables(
			panels.activeIssues.value
				.map(({ issue }) => issue.message.trim())
				.filter(Boolean)
				.join('\n\n'),
			legacy,
			current,
		)
	})
	watch(
		[() => project.value?.id, generated],
		([, message]) => {
			draft.value = message
		},
		{ immediate: true },
	)

	return { draft, generating }
}
