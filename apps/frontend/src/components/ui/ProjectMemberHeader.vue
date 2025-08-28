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
	<ModerationProjectNags
		v-if="
			(currentMember && project.status === 'draft') ||
			tags.rejectedStatuses.includes(project.status)
		"
		:project="project"
		:versions="versions"
		:current-member="currentMember"
		:collapsed="collapsed"
		:route-name="routeName"
		:tags="tags"
		@toggle-collapsed="handleToggleCollapsed"
		@set-processing="handleSetProcessing"
	/>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import type { Project, User, Version } from '@modrinth/utils'
import { defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

import { acceptTeamInvite, removeTeamMember } from '~/helpers/teams.js'

import ModerationProjectNags from './moderation/ModerationProjectNags.vue'

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
})

const { formatMessage } = useVIntl()

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
