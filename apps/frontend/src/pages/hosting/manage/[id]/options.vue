<template>
	<ServerSidebar :route="route" :nav-links="navLinks" :backup-in-progress="backupInProgress" />
</template>
<script setup lang="ts">
import {
	CardIcon,
	InfoIcon,
	ListIcon,
	ModrinthIcon,
	SettingsIcon,
	TextQuoteIcon,
	UserIcon,
	VersionIcon,
	WrenchIcon,
} from '@modrinth/assets'
import { injectModrinthServerContext } from '@modrinth/ui'
import { isAdmin as isUserAdmin, type User } from '@modrinth/utils'

import ServerSidebar from '~/components/ui/servers/ServerSidebar.vue'
import type { BackupInProgressReason } from '~/pages/hosting/manage/[id].vue'

const route = useRoute()
const serverId = route.params.id as string
const auth = await useAuth()

const { server } = injectModrinthServerContext()

defineProps<{
	backupInProgress?: BackupInProgressReason
}>()

useHead({
	title: `Options - ${server.value?.name ?? 'Server'} - Modrinth`,
})

const ownerId = computed(() => server.value?.owner_id ?? 'Ghost')
const isOwner = computed(() => (auth.value?.user as User | null)?.id === ownerId.value)
const isAdmin = computed(() => isUserAdmin(auth.value?.user))

const navLinks = computed(() => [
	{ icon: SettingsIcon, label: 'General', href: `/hosting/manage/${serverId}/options` },
	{ icon: WrenchIcon, label: 'Platform', href: `/hosting/manage/${serverId}/options/loader` },
	{ icon: TextQuoteIcon, label: 'Startup', href: `/hosting/manage/${serverId}/options/startup` },
	{ icon: VersionIcon, label: 'Network', href: `/hosting/manage/${serverId}/options/network` },
	{
		icon: ListIcon,
		label: 'Properties',
		href: `/hosting/manage/${serverId}/options/properties`,
		shown: server.value?.status !== 'installing',
	},
	{
		icon: UserIcon,
		label: 'Preferences',
		href: `/hosting/manage/${serverId}/options/preferences`,
	},
	{
		icon: CardIcon,
		label: 'Billing',
		href: `/settings/billing#server-${serverId}`,
		external: true,
		shown: isOwner.value,
	},
	{
		icon: ModrinthIcon,
		label: 'Admin Billing',
		href: `/admin/billing/${ownerId.value}`,
		external: true,
		shown: isAdmin.value,
	},
	{ icon: InfoIcon, label: 'Info', href: `/hosting/manage/${serverId}/options/info` },
])
</script>
