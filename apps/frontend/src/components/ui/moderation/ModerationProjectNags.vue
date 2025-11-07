<template>
	<div v-if="visibleNags.length > 0" class="universal-card my-4">
		<div class="flex max-w-full flex-wrap items-center gap-x-6 gap-y-4">
			<div class="flex flex-auto flex-wrap items-center gap-x-6 gap-y-4">
				<h2 class="my-0 mr-auto">
					{{ getFormattedMessage(messages.publishingChecklist) }}
				</h2>
				<div class="flex flex-row gap-2">
					<div class="flex items-center gap-1">
						<AsteriskIcon class="size-4 shrink-0 text-red" />
						<span class="text-secondary">{{ getFormattedMessage(messages.required) }}</span>
					</div>
					|
					<div class="flex items-center gap-1">
						<TriangleAlertIcon class="size-4 shrink-0 text-orange" />
						<span class="text-secondary">{{ getFormattedMessage(messages.warning) }}</span>
					</div>
					|
					<div class="flex items-center gap-1">
						<LightBulbIcon class="size-4 shrink-0 text-purple" />
						<span class="text-secondary">{{ getFormattedMessage(messages.suggestion) }}</span>
					</div>
				</div>
			</div>
			<div class="input-group">
				<ButtonStyled circular>
					<button :class="!collapsed && '[&>svg]:rotate-180'" @click="$emit('toggleCollapsed')">
						<DropdownIcon class="duration-250 transition-transform ease-in-out" />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div v-if="!collapsed" class="grid-display width-16 mt-4">
			<div v-for="nag in visibleNags" :key="nag.id" class="grid-display__item">
				<span class="flex items-center gap-2 font-semibold">
					<component
						:is="nag.icon || getDefaultIcon(nag.status)"
						v-tooltip="getStatusTooltip(nag.status)"
						:class="[
							'size-4',
							nag.status === 'required' && 'text-red',
							nag.status === 'warning' && 'text-orange',
							nag.status === 'suggestion' && 'text-purple',
						]"
						:aria-label="getStatusTooltip(nag.status)"
					/>
					{{ getFormattedMessage(nag.title) }}
				</span>
				{{ getNagDescription(nag) }}
				<NuxtLink
					v-if="nag.link && shouldShowLink(nag)"
					:to="`/${project.project_type}/${project.slug ? project.slug : project.id}/${
						nag.link.path
					}`"
					class="goto-link"
				>
					{{ getFormattedMessage(nag.link.title) }}
					<ChevronRightIcon aria-hidden="true" class="featured-header-chevron" />
				</NuxtLink>
				<ButtonStyled
					v-if="nag.status === 'special-submit-action' && nag.id === 'submit-for-review'"
					color="orange"
					@click="submitForReview"
				>
					<button
						v-tooltip="
							!canSubmitForReview ? getFormattedMessage(messages.submitChecklistTooltip) : undefined
						"
						:disabled="!canSubmitForReview"
					>
						<SendIcon />
						{{ getFormattedMessage(messages.submitForReviewButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	AsteriskIcon,
	ChevronRightIcon,
	DropdownIcon,
	LightBulbIcon,
	ScaleIcon,
	SendIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import type { Nag, NagContext, NagStatus } from '@modrinth/moderation'
import { nags } from '@modrinth/moderation'
import { ButtonStyled } from '@modrinth/ui'
import type { Project, User, Version } from '@modrinth/utils'
import { defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import type { Component } from 'vue'
import { computed } from 'vue'

interface Tags {
	rejectedStatuses: string[]
}

interface Member {
	accepted?: boolean
	project_role?: string
	user?: Partial<User>
}

interface Props {
	project: Project
	versions?: Version[]
	currentMember?: Member | null
	collapsed?: boolean
	routeName?: string
	tags: Tags
}

const messages = defineMessages({
	publishingChecklist: {
		id: 'project-moderation-nags.publishing-checklist',
		defaultMessage: 'Publishing checklist',
	},
	submitForReview: {
		id: 'project-moderation-nags.submit-for-review',
		defaultMessage: 'Submit for review',
	},
	submitForReviewDesc: {
		id: 'project-moderation-nags.submit-for-review-desc',
		defaultMessage:
			'Your project is only viewable by members of the project. It must be reviewed by moderators in order to be published.',
	},
	submitForReviewButton: {
		id: 'project-moderation-nags.submit-for-review-button',
		defaultMessage: 'Submit for review',
	},
	resubmitForReview: {
		id: 'project-moderation-nags.resubmit-for-review',
		defaultMessage: 'Resubmit for review',
	},
	resubmitForReviewDesc: {
		id: 'project-moderation-nags.resubmit-for-review-desc',
		defaultMessage:
			"Your project has been {status, select, rejected {rejected} withheld {withheld} other {{status}}} by Modrinth's staff. In most cases, you can resubmit for review after addressing the staff's message.",
	},
	visitModerationPage: {
		id: 'project-moderation-nags.visit-moderation-page',
		defaultMessage: 'Visit moderation page',
	},
	submitChecklistTooltip: {
		id: 'project-moderation-nags.submit-checklist-tooltip',
		defaultMessage: 'You must complete the required steps in the publishing checklist!',
	},
	required: {
		id: 'project-moderation-nags.required',
		defaultMessage: 'Required',
	},
	warning: {
		id: 'project-moderation-nags.warning',
		defaultMessage: 'Warning',
	},
	suggestion: {
		id: 'project-moderation-nags.suggestion',
		defaultMessage: 'Suggestion',
	},
})

const { formatMessage } = useVIntl()

const props = withDefaults(defineProps<Props>(), {
	versions: () => [],
	currentMember: null,
	collapsed: false,
	routeName: '',
})

const emit = defineEmits<{
	toggleCollapsed: []
	setProcessing: [processing: boolean]
}>()

const nagContext = computed<NagContext>(() => ({
	project: props.project,
	versions: props.versions,
	currentMember: props.currentMember as User,
	currentRoute: props.routeName,
	tags: props.tags,
	submitProject: submitForReview,
}))

const canSubmitForReview = computed(() => {
	return (
		applicableNags.value.filter((nag) => nag.status === 'required' && !isNagComplete(nag))
			.length === 0
	)
})

async function submitForReview() {
	if (canSubmitForReview.value) {
		emit('setProcessing', true)
	}
}

const applicableNags = computed<Nag[]>(() => {
	return nags.filter((nag) => {
		return nag.shouldShow(nagContext.value)
	})
})

function isNagComplete(nag: Nag): boolean {
	const context = nagContext.value
	return !nag.shouldShow(context)
}

const visibleNags = computed<Nag[]>(() => {
	const finalNags = applicableNags.value.filter((nag) => !isNagComplete(nag))

	if (props.project.status === 'draft') {
		finalNags.push({
			id: 'submit-for-review',
			title: messages.submitForReview,
			description: () => formatMessage(messages.submitForReviewDesc),
			status: 'special-submit-action',
			shouldShow: (ctx) => ctx.project.status === 'draft',
		})
	}

	if (props.tags.rejectedStatuses.includes(props.project.status)) {
		finalNags.push({
			id: 'resubmit-for-review',
			title: messages.resubmitForReview,
			description: (ctx) =>
				formatMessage(messages.resubmitForReviewDesc, { status: ctx.project.status }),
			status: 'special-submit-action',
			shouldShow: (ctx) => ctx.tags.rejectedStatuses.includes(ctx.project.status),
			link: {
				path: 'moderation',
				title: messages.visitModerationPage,
				shouldShow: () => props.routeName !== 'type-id-moderation',
			},
		})
	}

	finalNags.sort((a, b) => {
		const statusOrder = { required: 0, warning: 1, suggestion: 2, 'special-submit-action': 3 }
		return statusOrder[a.status] - statusOrder[b.status]
	})

	return finalNags
})

function shouldShowLink(nag: Nag): boolean {
	return nag.link?.shouldShow ? nag.link.shouldShow(nagContext.value) : false
}

function getDefaultIcon(status: NagStatus): Component {
	switch (status) {
		case 'required':
			return AsteriskIcon
		case 'warning':
			return TriangleAlertIcon
		case 'suggestion':
			return LightBulbIcon
		case 'special-submit-action':
			return ScaleIcon
		default:
			return AsteriskIcon
	}
}

function getStatusTooltip(status: NagStatus): string {
	switch (status) {
		case 'required':
			return formatMessage(messages.required)
		case 'warning':
			return formatMessage(messages.warning)
		case 'suggestion':
			return formatMessage(messages.suggestion)
		default:
			return formatMessage(messages.required)
	}
}

function getNagDescription(nag: Nag): string {
	if (typeof nag.description === 'function') {
		return nag.description(nagContext.value)
	}
	return formatMessage(nag.description)
}

function getFormattedMessage(message: string | MessageDescriptor): string {
	if (typeof message === 'string') {
		return message
	}
	return formatMessage(message)
}
</script>

<style lang="scss" scoped>
.duration-250 {
	transition-duration: 250ms;
}
</style>
