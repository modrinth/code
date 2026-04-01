<template>
	<div class="flex flex-col">
		<p class="mb-1 mt-0 font-bold">{{ formatMessage(commonSettingsMessages.sessions) }}</p>
		<p class="preserve-lines mb-2 mt-0">
			{{ formatMessage(messages.sessionsDescription) }}
		</p>
		<div
			v-for="session in sessions"
			:key="session.id"
			class="session mt-6 border-t border-divider first:mt-0 first:border-t-0 first:pt-0"
		>
			<div>
				<div>
					<strong>
						{{ session.os ?? formatMessage(messages.unknownOsLabel) }} ⋅
						{{ session.platform ?? formatMessage(messages.unknownPlatformLabel) }} ⋅
						{{ session.ip }}
					</strong>
				</div>
				<div>
					<template v-if="session.city">{{ session.city }}, {{ session.country }} ⋅ </template>
					<span v-tooltip="formatDateTime(session.last_login)">
						{{
							formatMessage(messages.lastAccessedAgoLabel, {
								ago: formatRelativeTime(session.last_login),
							})
						}}
					</span>
					⋅
					<span v-tooltip="formatDateTime(session.created)">
						{{
							formatMessage(messages.createdAgoLabel, {
								ago: formatRelativeTime(session.created),
							})
						}}
					</span>
				</div>
			</div>
			<div class="input-group">
				<i v-if="session.current">{{ formatMessage(messages.currentSessionLabel) }}</i>
				<ButtonStyled v-else color="red">
					<button type="button" class="iconified-button" @click="revokeSession(session.id)">
						{{ formatMessage(messages.revokeSessionButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>
<script setup>
import {
	ButtonStyled,
	commonMessages,
	commonSettingsMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

definePageMeta({
	middleware: 'auth',
})

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const messages = defineMessages({
	currentSessionLabel: {
		id: 'settings.sessions.current-session',
		defaultMessage: 'Current session',
	},
	revokeSessionButton: {
		id: 'settings.sessions.action.revoke-session',
		defaultMessage: 'Revoke session',
	},
	createdAgoLabel: {
		id: 'settings.sessions.created-ago',
		defaultMessage: 'Created {ago}',
	},
	sessionsDescription: {
		id: 'settings.sessions.description',
		defaultMessage:
			"Here are all the devices that are currently logged in with your Modrinth account. You can log out of each one individually.\n\nIf you see an entry you don't recognize, log out of that device and change your Modrinth account password immediately.",
	},
	lastAccessedAgoLabel: {
		id: 'settings.sessions.last-accessed-ago',
		defaultMessage: 'Last accessed {ago}',
	},
	unknownOsLabel: {
		id: 'settings.sessions.unknown-os',
		defaultMessage: 'Unknown OS',
	},
	unknownPlatformLabel: {
		id: 'settings.sessions.unknown-platform',
		defaultMessage: 'Unknown platform',
	},
})

useHead({
	title: () => `${formatMessage(commonSettingsMessages.sessions)} - Modrinth`,
})

const { data: sessions } = useQuery({
	queryKey: ['session', 'list'],
	queryFn: () => client.labrinth.sessions_v2.list(),
})

async function revokeSession(id) {
	startLoading()
	try {
		sessions.value = sessions.value.filter((x) => x.id !== id)
		await client.labrinth.sessions_v2.delete(id)
		await queryClient.invalidateQueries({ queryKey: ['session', 'list'] })
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}
</script>
<style lang="scss" scoped>
.session {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;

	@media screen and (min-width: 800px) {
		flex-direction: row;
		align-items: center;

		.input-group {
			margin-left: auto;
		}
	}
}
</style>
