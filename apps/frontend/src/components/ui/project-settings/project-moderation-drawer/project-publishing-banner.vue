<script setup lang="ts">
import { ChevronRightIcon, Clock4Icon, SendIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { Admonition, Button, defineMessage, IconButton, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import {
	projectPublishingColors,
	type ProjectPublishingState,
	projectPublishingTones,
} from '~/providers/project-publishing'

const props = withDefaults(
	defineProps<{
		state: ProjectPublishingState
		compact?: boolean
		remaining?: number
		completed?: number
		expanded?: boolean
		controls?: string
		submitting?: boolean
	}>(),
	{ compact: false, remaining: 0, expanded: false, controls: undefined },
)
defineEmits<{ open: []; submit: []; dismiss: [] }>()
const { formatMessage } = useVIntl()
const dismissLabel = defineMessage({
	id: 'project.publishing.dismiss-banner',
	defaultMessage: 'Dismiss publishing banner',
})
const submitAction = computed(() => {
	switch (props.state) {
		case 'ready':
			return defineMessage({
				id: 'project.publishing.ready.action',
				defaultMessage: 'Submit for review',
			})
		case 'ready-to-resubmit':
			return defineMessage({
				id: 'project.publishing.resubmit.action',
				defaultMessage: 'Resubmit for review',
			})
	}
})
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
		default:
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
						'Your required checks are complete. Submit your project and the moderation team will review it.',
				}),
			}
		case 'ready-to-resubmit':
			return {
				title: defineMessage({
					id: 'project.publishing.resubmit.title',
					defaultMessage: 'Ready to resubmit',
				}),
				body: defineMessage({
					id: 'project.publishing.resubmit.body',
					defaultMessage:
						'Thanks for making those changes. Resubmit your project and the moderation team will take another look.',
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
						'The content moderation team is reviewing your project. Most reviews take under a week. You can still make changes without losing your place in the queue.',
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
									defaultMessage: 'Your project is published and discoverable on Modrinth.',
								}),
			}
		case 'withheld':
			return {
				title: defineMessage({
					id: 'project.publishing.withheld.title',
					defaultMessage: 'Project approved with limited visibility',
				}),
				body: defineMessage({
					id: 'project.publishing.withheld.body',
					defaultMessage:
						'Your project can only be accessed with a direct link. If you’d like to resubmit for full visibility, review the moderator feedback and address any requested changes.',
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
	}
})
const tone = computed(() => projectPublishingTones[props.state])
const buttonColor = computed(() => projectPublishingColors[tone.value])
const action = computed(() =>
	formatMessage(
		props.state === 'todo'
			? defineMessage({ id: 'project.publishing.checklist', defaultMessage: 'View checklist' })
			: defineMessage({ id: 'project.publishing.details', defaultMessage: 'View details' }),
	),
)
const checklistProgress = computed(() => {
	const { state, completed, remaining } = props
	if (completed === undefined || remaining <= 0 || (state !== 'todo' && state !== 'changes')) return
	return { completed, total: completed + remaining }
})
const summaryCopy = computed(() => {
	if (props.state === 'changes' && checklistProgress.value) {
		return {
			title: defineMessage({
				id: 'project.publishing.changes.checklist.title',
				defaultMessage: '{count, plural, one {# item} other {# items}} left to resubmit',
			}),
		}
	}
	if (props.state === 'review') {
		return {
			title: copy.value.title,
			body: defineMessage({
				id: 'project.publishing.review.summary',
				defaultMessage:
					'Moderators are reviewing your project. Most reviews take under a week. If anything else needs changing, we’ll message you in the thread.',
			}),
		}
	}
	return copy.value
})
const checklistProgressLabel = defineMessage({
	id: 'project.publishing.checklist.progress',
	defaultMessage: 'Review checklist progress',
})
</script>

<template>
	<div v-if="compact">
		<h2 class="m-0 text-base font-semibold text-contrast">
			{{ formatMessage(summaryCopy.title, { count: remaining }) }}
		</h2>
		<p
			v-if="state !== 'todo' && summaryCopy.body"
			class="mb-0 mt-1 text-sm leading-relaxed text-secondary"
		>
			{{ formatMessage(summaryCopy.body) }}
		</p>
		<div
			v-if="checklistProgress"
			role="progressbar"
			:aria-label="formatMessage(checklistProgressLabel)"
			aria-valuemin="0"
			:aria-valuemax="checklistProgress.total"
			:aria-valuenow="checklistProgress.completed"
			class="mt-3 h-1.5 overflow-hidden rounded-full bg-surface-4"
		>
			<div
				class="h-full rounded-full bg-orange"
				:style="{ width: `${(checklistProgress.completed / checklistProgress.total) * 100}%` }"
			/>
		</div>
	</div>
	<Admonition
		v-else
		:type="tone"
		:header="formatMessage(copy.title, { count: remaining })"
		:body="formatMessage(copy.body)"
		inline-actions
		center-content
	>
		<template v-if="state === 'checking' || state === 'review'" #icon="{ iconClass }">
			<SpinnerIcon
				v-if="state === 'checking'"
				class="size-6 shrink-0 animate-spin text-secondary"
				aria-hidden="true"
			/>
			<Clock4Icon v-else :class="iconClass" aria-hidden="true" />
		</template>
		<template #actions>
			<Button
				v-if="submitAction"
				type="colored"
				:color="buttonColor"
				:loading="submitting"
				@click="$emit('submit')"
			>
				<SendIcon /> {{ formatMessage(submitAction) }}
			</Button>
			<Button
				v-else
				:type="buttonColor ? 'colored' : 'base'"
				:color="buttonColor"
				:aria-controls="controls"
				:aria-expanded="expanded"
				@click="$emit('open')"
			>
				{{ action }} <ChevronRightIcon />
			</Button>
			<IconButton
				v-if="state === 'approved' || state === 'withheld'"
				class="ml-2"
				type="quiet"
				:color="buttonColor"
				:circular="false"
				:label="formatMessage(dismissLabel)"
				@click="$emit('dismiss')"
			>
				<XIcon aria-hidden="true" />
			</IconButton>
		</template>
	</Admonition>
</template>
