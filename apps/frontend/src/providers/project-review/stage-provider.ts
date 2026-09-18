import type { Labrinth } from '@modrinth/api-client'
import { type ProjectPageContext, provideProjectPageContext } from '@modrinth/ui'
import { computed, defineComponent, h, onScopeDispose, type PropType } from 'vue'

import { injectProjectReviewPageContext } from './index'
import { injectReviewStages, useReviewStageDefinitions } from './review-stages'

export default defineComponent({
	name: 'ReviewStageProvider',
	inheritAttrs: false,
	props: {
		project: {
			type: Object as PropType<Labrinth.Projects.v3.Project>,
			required: true,
		},
	},
	setup(props, { slots }) {
		const { projectV2, threadQuery } = injectProjectReviewPageContext()
		const stageProject = {
			projectV3: computed(() => props.project),
			projectV2,
			thread: threadQuery.data,
		}
		provideProjectPageContext(stageProject as ProjectPageContext)

		const StageScope = defineComponent({
			name: 'ReviewStageScope',
			setup(_, { slots }) {
				const { register } = injectReviewStages()
				onScopeDispose(register(props.project.id, useReviewStageDefinitions()))
				return () => slots.default?.()
			},
		})

		return () => h(StageScope, null, slots)
	},
})
