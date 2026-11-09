<script setup lang="ts">
import { ChevronRightIcon, SpinnerIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessage, useVIntl } from '../../composables'
import Admonition from '../base/Admonition.vue'
import Button from '../base/buttons/Button.vue'

const props = withDefaults(
	defineProps<{
		state:
			| 'checking'
			| 'unavailable'
			| 'todo'
			| 'ready'
			| 'review'
			| 'approved'
			| 'unlisted'
			| 'private'
			| 'withheld'
			| 'changes'
			| 'default'
		compact?: boolean
		remaining?: number
		expanded?: boolean
		controls?: string
	}>(),
	{ compact: false, remaining: 0, expanded: false, controls: undefined },
)
defineEmits<{ open: [] }>()
const { formatMessage } = useVIntl()
const copy = computed(() => {
	switch (props.state) {
		case 'checking':
			return {
				title: defineMessage({
					id: 'project.publishing.checking.title',
					defaultMessage: 'Checking your project',
				}),
				body: defineMessage({
					id: 'project.publishing.checking.body',
					defaultMessage: 'Checking your latest saved changes.',
				}),
			}
		case 'unavailable':
			return {
				title: defineMessage({
					id: 'project.publishing.unavailable.title',
					defaultMessage: 'Checks unavailable',
				}),
				body: defineMessage({
					id: 'project.publishing.unavailable.body',
					defaultMessage: 'Reload the page to try the project checks again.',
				}),
			}
		case 'todo':
			return {
				title: defineMessage({
					id: 'project.publishing.todo.title',
					defaultMessage: '{count, plural, one {# item} other {# items}} left before review',
				}),
				body: defineMessage({
					id: 'project.publishing.todo.body',
					defaultMessage: 'Complete these items to submit your project for review.',
				}),
			}
		case 'ready':
			return {
				title: defineMessage({
					id: 'project.publishing.ready.title',
					defaultMessage: 'Ready for review',
				}),
				body: defineMessage({
					id: 'project.publishing.ready.body',
					defaultMessage:
						'Your required checks are complete. Open the checklist to submit your project for review.',
				}),
			}
		case 'review':
			return {
				title: defineMessage({
					id: 'project.publishing.review.title',
					defaultMessage: 'Project under review',
				}),
				body: defineMessage({
					id: 'project.publishing.review.body',
					defaultMessage:
						'The content moderation team is reviewing your project. Most reviews take under a week. You can still make changes without losing your place in the queue. If anything needs changing, we’ll message you in the thread.',
				}),
			}
		case 'approved':
		case 'unlisted':
		case 'private':
			return {
				title: defineMessage({
					id: 'project.publishing.approved.title',
					defaultMessage: 'Project approved',
				}),
				body:
					props.state === 'private'
						? defineMessage({
								id: 'project.publishing.private.body',
								defaultMessage:
									'Your project is private and can only be accessed by you and people you invite. You can change this in your visibility settings.',
							})
						: props.state === 'unlisted'
							? defineMessage({
									id: 'project.publishing.unlisted.body',
									defaultMessage:
										'Your project is unlisted and can only be accessed with a direct link. You can change this in your visibility settings.',
								})
							: defineMessage({
									id: 'project.publishing.approved.body',
									defaultMessage:
										'Your project is published and discoverable on Modrinth. You can change this in your visibility settings.',
								}),
			}
		case 'withheld':
			return {
				title: defineMessage({
					id: 'project.publishing.withheld.title',
					defaultMessage: 'Unlisted by staff',
				}),
				body: defineMessage({
					id: 'project.publishing.withheld.body',
					defaultMessage:
						'Your project can only be accessed with a direct link. Review the moderator feedback and address any requested changes before resubmitting. If you requested unlisted visibility, no action may be necessary.',
				}),
			}
		case 'changes':
			return {
				title: defineMessage({
					id: 'project.publishing.changes.title',
					defaultMessage: 'Changes requested',
				}),
				body: defineMessage({
					id: 'project.publishing.changes.body',
					defaultMessage:
						'Review the moderator feedback and address all requested changes before resubmitting your project.',
				}),
			}
		default:
			return {
				title: defineMessage({
					id: 'project.publishing.default.title',
					defaultMessage: 'Review and publishing',
				}),
				body: defineMessage({
					id: 'project.publishing.default.body',
					defaultMessage: 'Manage project checks and your conversation with moderators.',
				}),
			}
	}
})
const tone = computed(() => {
	if (['review', 'withheld'].includes(props.state)) return 'info'
	if (['approved', 'unlisted', 'private'].includes(props.state)) return 'success'
	if (['todo', 'changes'].includes(props.state)) return 'warning'
	return 'neutral'
})
const buttonColor = computed(
	() =>
		(({ info: 'blue', success: 'green', warning: 'orange', neutral: undefined }) as const)[
			tone.value
		],
)
const action = computed(() =>
	formatMessage(
		props.state === 'todo' ||
			props.state === 'ready' ||
			props.state === 'checking' ||
			props.state === 'unavailable'
			? defineMessage({ id: 'project.publishing.checklist', defaultMessage: 'View checklist' })
			: defineMessage({ id: 'project.publishing.details', defaultMessage: 'View details' }),
	),
)
const summaryBody = computed(() =>
	props.state === 'review'
		? defineMessage({
				id: 'project.publishing.review.summary',
				defaultMessage:
					'Moderators are reviewing your project. Most reviews take under a week. If anything else needs changing, we’ll message you in the thread.',
			})
		: copy.value.body,
)
</script>

<template>
	<div v-if="compact">
		<p class="m-0 text-base font-semibold text-contrast">
			{{ formatMessage(copy.title, { count: remaining }) }}
		</p>
		<p class="mb-0 mt-1 text-sm leading-relaxed text-secondary">
			{{ formatMessage(summaryBody) }}
		</p>
	</div>
	<Admonition
		v-else
		:type="tone"
		:header="formatMessage(copy.title, { count: remaining })"
		:body="formatMessage(copy.body)"
		inline-actions
		center-content
	>
		<template v-if="state === 'checking'" #icon>
			<SpinnerIcon class="size-6 shrink-0 animate-spin text-secondary" aria-hidden="true" />
		</template>
		<template #actions>
			<Button
				:type="buttonColor ? 'colored' : 'base'"
				:color="buttonColor"
				:aria-controls="controls"
				:aria-expanded="expanded"
				@click="$emit('open')"
			>
				{{ action }} <ChevronRightIcon />
			</Button>
		</template>
	</Admonition>
</template>
