import type { Labrinth } from '@modrinth/api-client'
import {
	collectActiveActions,
	evalActiveAction,
	isShown,
	resolveChildren,
	setMessageProject,
	type Prioritizable,
} from '@modrinth/moderation/src/types/node'
import { expandVariables } from '@modrinth/moderation/src/utils'
import {
	injectNotificationManager,
	type ProjectPageContext,
	provideProjectPageContext,
} from '@modrinth/ui'
import { computed, defineComponent, h, onScopeDispose, type PropType, watch } from 'vue'

import { injectProjectReviewPageContext } from './index'
import { injectReviewSession } from './review-session'
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
				const { register, draft, generating } = injectReviewStages()
				const session = injectReviewSession()
				const { addNotification } = injectNotificationManager()
				const stages = useReviewStageDefinitions()
				onScopeDispose(register(props.project.id, stages))
				watch(
					[() => session.readProject(props.project.id), computed(() => !!projectV2.value)],
					async ([state], _, onCleanup) => {
						let cancelled = false
						onCleanup(() => {
							cancelled = true
						})
						draft.value = ''
						if (!projectV2.value) {
							generating.value = false
							return
						}
						generating.value = true
						try {
							setMessageProject(
								stageProject.projectV3,
								computed(() => projectV2.value!),
							)
							const actions = Object.values(stages)
								.filter(isShown)
								.flatMap((stage) => {
									const localState = state[stage.id] ?? {}
									const entries = collectActiveActions(
										resolveChildren(stage, localState),
										localState,
										[stage.id],
									)
									if (stage._segments.length)
										entries.unshift({
											node: stage,
											state: localState,
											statePath: [stage.id],
											active: true,
										})
									return entries
								})
							const consumed = new Set<object>()
							const parts = []
							for (const entry of actions) {
								if (cancelled) return
								if (consumed.has(entry.node)) continue
								const content = await evalActiveAction(entry, actions, consumed)
								if (content.trim()) parts.push({ entry, content })
							}
							parts.sort((a, b) =>
								(a.entry.node as Prioritizable)._priority.compareTo(
									(b.entry.node as Prioritizable)._priority,
								),
							)
							if (!cancelled)
								draft.value = expandVariables(
									parts.map(({ content }) => content.trim()).join('\n\n'),
									projectV2.value,
									props.project,
								)
						} catch (error) {
							if (!cancelled)
								addNotification({
									title: 'Error generating message',
									text: error instanceof Error ? error.message : String(error),
									type: 'error',
								})
						} finally {
							if (!cancelled) generating.value = false
						}
					},
					{ deep: true, immediate: true },
				)
				return () => slots.default?.()
			},
		})

		return () => h(StageScope, null, slots)
	},
})
