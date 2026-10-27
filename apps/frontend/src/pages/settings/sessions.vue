<template>
	<div class="flex flex-col">
		<h2 class="m-0 text-2xl font-semibold">{{ formatMessage(commonSettingsMessages.sessions) }}</h2>
		<p class="my-2 leading-tight">
			{{ formatMessage(messages.sessionsDescription) }}
		</p>
		<p class="mb-4 mt-0 leading-tight">
			{{ formatMessage(messages.sessionsDescription2) }}
		</p>
		<template v-if="isLoading">
			<div class="flex flex-col gap-3">
				<div
					v-for="i in 2"
					:key="`session-skeleton-${i}`"
					class="flex flex-wrap items-center gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
				>
					<div class="size-12 shrink-0 animate-pulse rounded-xl bg-surface-4" />
					<div class="flex min-w-0 flex-1 flex-col gap-2">
						<div class="h-4 w-40 animate-pulse rounded-lg bg-surface-4" />
						<div class="h-3 w-72 max-w-full animate-pulse rounded-lg bg-surface-4" />
					</div>
					<div class="h-9 w-32 animate-pulse rounded-xl bg-surface-4" />
				</div>
			</div>
		</template>
		<EmptyState
			v-else-if="displaySessions.length === 0"
			type="empty"
			:heading="formatMessage(messages.emptyStateHeading)"
			:description="formatMessage(messages.emptyStateDescription)"
		/>
		<div v-else class="flex flex-col gap-3">
			<div
				v-for="session in displaySessions"
				:key="session.id"
				class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-solid bg-surface-3 p-4"
				:class="session.current ? 'border-green' : 'border-surface-4'"
			>
				<div class="flex min-w-0 flex-1 items-center gap-3">
					<img
						v-if="isModrinthAppSession(session)"
						src="https://cdn.modrinth.com/mr-social-icon.jpg"
						alt=""
						class="mb-auto size-12 shrink-0 rounded-xl border border-solid border-white/10"
					/>
					<div
						v-else
						class="mb-auto flex size-12 shrink-0 items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-4 text-contrast"
					>
						<component :is="sessionOsIcon(session.os)" class="size-6" />
					</div>
					<div class="flex min-w-0 flex-col gap-0.5">
						<div class="flex min-w-0 flex-wrap items-center gap-2">
							<h2 class="m-0 break-words text-base font-semibold">
								{{ sessionTitle(session) }}
							</h2>
							<span
								v-if="session.current"
								class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-highlight-green px-2.5 py-0.5 text-sm font-medium text-green"
							>
								<span class="size-1.5 rounded-full bg-green" aria-hidden="true" />
								{{ formatMessage(messages.currentSessionLabel) }}
							</span>
						</div>
						<PageHeaderMetadata>
							<PageHeaderMetadataItem v-if="sessionLocation(session)">
								{{ sessionLocation(session) }}
							</PageHeaderMetadataItem>
							<PageHeaderMetadataItem :tooltip="formatDateTime(session.last_login)">
								{{
									formatMessage(messages.lastAccessedAgoLabel, {
										ago: formatRelativeTime(session.last_login),
									})
								}}
							</PageHeaderMetadataItem>
							<PageHeaderMetadataItem :tooltip="formatDateTime(session.created)">
								{{
									formatMessage(messages.createdAgoLabel, {
										ago: formatRelativeTime(session.created),
									})
								}}
							</PageHeaderMetadataItem>
						</PageHeaderMetadata>
						<div class="mt-1 text-sm text-secondary opacity-75">
							{{ formatMessage(messages.ipLabel, { ip: session.ip }) }}
						</div>
					</div>
				</div>
				<Button
					v-if="!session.current"
					type="colored"
					color="red"
					@click="revokeSession(session.id)"
				>
					<XCircleIcon />
					{{ formatMessage(messages.revokeSessionButton) }}
				</Button>
			</div>
		</div>
	</div>
</template>
<script setup>
import {
	MonitorIcon,
	MonitorSmartphoneIcon,
	SmartphoneIcon,
	TabletIcon,
	XCircleIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	commonSettingsMessages,
	defineMessages,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
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
		defaultMessage: 'Signed in {ago}',
	},
	sessionsDescription: {
		id: 'settings.sessions.description.1',
		defaultMessage: `These are all the devices currently signed in with your Modrinth account. You can sign out of each one individually by revoking their session. If you don't recognize a device, sign out of that device and change your Modrinth account password immediately.`,
	},
	sessionsDescription2: {
		id: 'settings.sessions.description.2',
		defaultMessage:
			'Locations listed may be approximate since they are based on the IP address. VPNs and some internet service providers may obscure the actual location.',
	},
	lastAccessedAgoLabel: {
		id: 'settings.sessions.last-accessed-ago',
		defaultMessage: 'Last accessed {ago}',
	},
	ipLabel: {
		id: 'settings.sessions.ip',
		defaultMessage: 'IP address: {ip}',
	},
	unknownOsLabel: {
		id: 'settings.sessions.unknown-os',
		defaultMessage: 'Unknown OS',
	},
	unknownPlatformLabel: {
		id: 'settings.sessions.unknown-platform',
		defaultMessage: 'Unknown platform',
	},
	modrinthAppLabel: {
		id: 'settings.sessions.modrinth-app',
		defaultMessage: 'Modrinth App',
	},
	emptyStateHeading: {
		id: 'settings.sessions.empty-state.heading',
		defaultMessage: 'No active sessions',
	},
	emptyStateDescription: {
		id: 'settings.sessions.empty-state.description',
		defaultMessage: 'Devices signed in to your account will appear here.',
	},
})

useHead({
	title: () => `${formatMessage(commonSettingsMessages.sessions)} - Modrinth`,
})

const { data: sessions, isLoading } = useQuery({
	queryKey: ['session', 'list'],
	queryFn: () => client.labrinth.sessions_v2.list(),
})

const displaySessions = computed(() => {
	return [...(sessions.value ?? [])].sort((a, b) => {
		if (a.current !== b.current) {
			return a.current ? -1 : 1
		}
		return new Date(b.last_login).getTime() - new Date(a.last_login).getTime()
	})
})

const sessionOperatingSystems = {
	'Windows 10': { label: 'Windows', icon: MonitorIcon },
	'Windows 8.1': { icon: MonitorIcon },
	'Windows 8': { icon: MonitorIcon },
	'Windows 7': { icon: MonitorIcon },
	'Windows Vista': { icon: MonitorIcon },
	'Windows XP': { icon: MonitorIcon },
	'Windows 2000': { icon: MonitorIcon },
	'Windows NT 4.0': { icon: MonitorIcon },
	'Windows 98': { icon: MonitorIcon },
	'Windows 95': { icon: MonitorIcon },
	'Windows UNKNOWN Ver': { label: 'Windows', icon: MonitorIcon },
	'Mac OSX': { label: 'macOS', icon: MonitorIcon },
	'Mac OS Classic': { label: 'macOS', icon: MonitorIcon },
	ChromeOS: { icon: MonitorIcon },
	Linux: { icon: MonitorIcon },
	BSD: { icon: MonitorIcon },
	iPhone: { icon: SmartphoneIcon },
	iPod: { icon: SmartphoneIcon },
	iOS: { icon: SmartphoneIcon },
	Android: { icon: SmartphoneIcon },
	'Windows Phone OS': { icon: SmartphoneIcon },
	'Windows CE': { icon: SmartphoneIcon },
	'Firefox OS': { icon: SmartphoneIcon },
	BlackBerry: { icon: SmartphoneIcon },
	'BlackBerry 10': { icon: SmartphoneIcon },
	docomo: { icon: SmartphoneIcon },
	au: { icon: SmartphoneIcon },
	SoftBank: { icon: SmartphoneIcon },
	WILLCOM: { icon: SmartphoneIcon },
	jig: { icon: SmartphoneIcon },
	emobile: { icon: SmartphoneIcon },
	SymbianOS: { icon: SmartphoneIcon },
	iPad: { icon: TabletIcon },
}

function sessionOsLabel(os) {
	if (!os || os === 'UNKNOWN') {
		return formatMessage(messages.unknownOsLabel)
	}
	return sessionOperatingSystems[os]?.label ?? os
}

function sessionOsIcon(os) {
	return sessionOperatingSystems[os]?.icon ?? MonitorSmartphoneIcon
}

function isModrinthAppSession(session) {
	return session.user_agent?.startsWith('modrinth/theseus/')
}

const modrinthAppOperatingSystems = {
	macos: 'macOS',
	windows: 'Windows',
	linux: 'Linux',
	android: 'Android',
	ios: 'iOS',
}

function modrinthAppOsLabel(userAgent) {
	const match = userAgent.match(/^modrinth\/theseus\/\S+ \(([^;)]+)/)
	const os = match?.[1]?.trim().toLowerCase()
	if (!os || os.includes('@')) {
		return null
	}
	return modrinthAppOperatingSystems[os] ?? os
}

function sessionTitle(session) {
	if (isModrinthAppSession(session)) {
		const app = formatMessage(messages.modrinthAppLabel)
		const os = modrinthAppOsLabel(session.user_agent)
		return os ? `${os} · ${app}` : app
	}

	const platform = session.platform ?? formatMessage(messages.unknownPlatformLabel)
	return `${sessionOsLabel(session.os)} · ${platform}`
}

function sessionLocation(session) {
	return [session.city, session.country].filter(Boolean).join(', ')
}

async function revokeSession(id) {
	startLoading()
	const previous = queryClient.getQueryData(['session', 'list'])
	queryClient.setQueryData(['session', 'list'], (old) =>
		(old ?? []).filter((session) => session.id !== id),
	)
	try {
		await client.labrinth.sessions_v2.delete(id)
	} catch (err) {
		queryClient.setQueryData(['session', 'list'], previous)
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}
</script>
