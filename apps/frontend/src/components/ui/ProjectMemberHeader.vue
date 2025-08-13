<template>
	<div v-if="showInvitation" class="universal-card information invited my-4">
		<h2>{{ getFormattedMessage(messages.invitationTitle) }}</h2>
		<p v-if="currentMember?.project_role">
			{{ formatMessage(messages.invitationWithRole, { role: currentMember.project_role }) }}
		</p>
		<p v-else>{{ getFormattedMessage(messages.invitationNoRole) }}</p>
		<div class="input-group">
			<ButtonStyled color="brand">
				<button class="brand-button" @click="acceptInvite()">
					<CheckIcon />
					{{ getFormattedMessage(messages.accept) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="red">
				<button @click="declineInvite">
					<XIcon />
					{{ getFormattedMessage(messages.decline) }}
				</button>
			</ButtonStyled>
		</div>
	</div>
	<div
		v-if="
			currentMember &&
			visibleNags.length > 0 &&
			(project.status === 'draft' || tags.rejectedStatuses.includes(project.status))
		"
		class="universal-card my-4"
	>
		<div class="flex max-w-full flex-wrap items-center gap-x-6 gap-y-4">
			<div class="flex flex-auto flex-wrap items-center gap-x-6 gap-y-4">
				<h2 class="my-0 mr-auto">
					{{ getFormattedMessage(messages.publishingChecklist) }}
				</h2>
				<div class="flex flex-row gap-2">
					<div class="flex items-center gap-1">
						<AsteriskIcon class="size-4 text-red" />
						<span class="text-secondary">{{ getFormattedMessage(messages.required) }}</span>
					</div>
					|
					<div class="flex items-center gap-1">
						<TriangleAlertIcon class="size-4 text-orange" />
						<span class="text-secondary">{{ getFormattedMessage(messages.warning) }}</span>
					</div>
					|
					<div class="flex items-center gap-1">
						<LightBulbIcon class="size-4 text-purple" />
						<span class="text-secondary">{{ getFormattedMessage(messages.suggestion) }}</span>
					</div>
				</div>
			</div>
			<div class="input-group">
				<ButtonStyled circular>
					<button :class="!collapsed && '[&>svg]:rotate-180'" @click="handleToggleCollapsed()">
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
						{{ getFormattedMessage(messages.submitForReview) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	AsteriskIcon,
	CheckIcon,
	ChevronRightIcon,
	DropdownIcon,
	LightBulbIcon,
	ScaleIcon,
	SendIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import type { Nag, NagContext, NagStatus } from '@modrinth/moderation'
import { nags } from '@modrinth/moderation'
import { ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import type { Project, User, Version } from '@modrinth/utils'
import { defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import type { Component } from 'vue'
import { computed } from 'vue'

import { acceptTeamInvite, removeTeamMember } from '~/helpers/teams.js'

const { addNotification } = injectNotificationManager()

interface Tags {
	rejectedStatuses: string[]
}

interface Auth {
	user: {
		id: string
	}
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
	allMembers?: Member[] | null
	isSettings?: boolean
	collapsed?: boolean
	routeName?: string
	auth: Auth
	tags: Tags
	setProcessing?: (processing: boolean) => void
	toggleCollapsed?: () => void
	updateMembers?: () => void | Promise<void>
}

const messages = defineMessages({
	invitationTitle: {
		id: 'project-member-header.invitation-title',
		defaultMessage: 'Invitation to join project',
	},
	invitationWithRole: {
		id: 'project-member-header.invitation-with-role',
		defaultMessage: "You've been invited be a member of this project with the role of '{role}'.",
	},
	invitationNoRole: {
		id: 'project-member-header.invitation-no-role',
		defaultMessage:
			"You've been invited to join this project. Please accept or decline the invitation.",
	},
	accept: {
		id: 'project-member-header.accept',
		defaultMessage: 'Accept',
	},
	decline: {
		id: 'project-member-header.decline',
		defaultMessage: 'Decline',
	},
	publishingChecklist: {
		id: 'project-member-header.publishing-checklist',
		defaultMessage: 'Publishing checklist',
	},
	submitForReview: {
		id: 'project-member-header.submit-for-review',
		defaultMessage: 'Submit for review',
	},
	submitForReviewDesc: {
		id: 'project-member-header.submit-for-review-desc',
		defaultMessage:
			'Your project is only viewable by members of the project. It must be reviewed by moderators in order to be published.',
	},
	resubmitForReview: {
		id: 'project-member-header.resubmit-for-review',
		defaultMessage: 'Resubmit for review',
	},
	resubmitForReviewDesc: {
		id: 'project-member-header.resubmit-for-review-desc',
		defaultMessage:
			"Your project has been {status} by Modrinth's staff. In most cases, you can resubmit for review after addressing the staff's message.",
	},
	showKey: {
		id: 'project-member-header.show-key',
		defaultMessage: 'Toggle key',
	},
	keyTitle: {
		id: 'project-member-header.key-title',
		defaultMessage: 'Status Key',
	},
	action: {
		id: 'project-member-header.action',
		defaultMessage: 'Action',
	},
	visitModerationPage: {
		id: 'project-member-header.visit-moderation-page',
		defaultMessage: 'Visit moderation page',
	},
	submitChecklistTooltip: {
		id: 'project-member-header.submit-checklist-tooltip',
		defaultMessage: 'You must complete the required steps in the publishing checklist!',
	},
	successJoin: {
		id: 'project-member-header.success-join',
		defaultMessage: 'You have joined the project team',
	},
	errorJoin: {
		id: 'project-member-header.error-join',
		defaultMessage: 'Failed to accept team invitation',
	},
	successDecline: {
		id: 'project-member-header.success-decline',
		defaultMessage: 'You have declined the team invitation',
	},
	errorDecline: {
		id: 'project-member-header.error-decline',
		defaultMessage: 'Failed to decline team invitation',
	},
	success: {
		id: 'project-member-header.success',
		defaultMessage: 'Success',
	},
	error: {
		id: 'project-member-header.error',
		defaultMessage: 'Error',
	},
	required: {
		id: 'project-member-header.required',
		defaultMessage: 'Required',
	},
	warning: {
		id: 'project-member-header.warning',
		defaultMessage: 'Warning',
	},
	suggestion: {
		id: 'project-member-header.suggestion',
		defaultMessage: 'Suggestion',
	},
})

const { formatMessage } = useVIntl()

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

const props = withDefaults(defineProps<Props>(), {
	versions: () => [],
	currentMember: null,
	allMembers: null,
	isSettings: false,
	collapsed: false,
	routeName: '',
	setProcessing: () => {},
	toggleCollapsed: () => {},
	updateMembers: async () => {},
})

const emit = defineEmits<{
	toggleCollapsed: []
	updateMembers: []
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
		await handleSetProcessing(true)
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

const showInvitation = computed<boolean>(() => {
	if (props.allMembers && props.auth) {
		const member = props.allMembers.find((x) => x?.user?.id === props.auth.user.id)
		return !!member && !member.accepted
	}
	return false
})

function handleToggleCollapsed(): void {
	if (props.toggleCollapsed) {
		props.toggleCollapsed()
	} else {
		emit('toggleCollapsed')
	}
}

async function handleUpdateMembers(): Promise<void> {
	if (props.updateMembers) {
		await props.updateMembers()
	} else {
		emit('updateMembers')
	}
}

function handleSetProcessing(processing: boolean): void {
	if (props.setProcessing) {
		props.setProcessing(processing)
	} else {
		emit('setProcessing', processing)
	}
}

async function acceptInvite(): Promise<void> {
	try {
		handleSetProcessing(true)
		await acceptTeamInvite(props.project.team)
		await handleUpdateMembers()
		addNotification({
			title: formatMessage(messages.success),
			text: formatMessage(messages.successJoin),
			type: 'success',
		})
	} catch {
		addNotification({
			title: formatMessage(messages.error),
			text: formatMessage(messages.errorJoin),
			type: 'error',
		})
	} finally {
		handleSetProcessing(false)
	}
}

async function declineInvite(): Promise<void> {
	try {
		handleSetProcessing(true)
		await removeTeamMember(props.project.team, props.auth.user.id)
		await handleUpdateMembers()
		addNotification({
			title: formatMessage(messages.success),
			text: formatMessage(messages.successDecline),
			type: 'success',
		})
	} catch {
		addNotification({
			title: formatMessage(messages.error),
			text: formatMessage(messages.errorDecline),
			type: 'error',
		})
	} finally {
		handleSetProcessing(false)
	}
}
</script>

<style lang="scss" scoped>
.duration-250 {
	transition-duration: 250ms;
}
</style>
