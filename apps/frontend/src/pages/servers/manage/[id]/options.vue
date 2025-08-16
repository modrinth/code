<template>
	<ServerSidebar
		:route="route"
		:nav-links="navLinks"
		:server="server"
		:backup-in-progress="backupInProgress"
	/>
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
import { isAdmin as isUserAdmin, type User } from '@modrinth/utils'

import ServerSidebar from '~/components/ui/servers/ServerSidebar.vue'
import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'
import type { BackupInProgressReason } from '~/pages/servers/manage/[id].vue'

const route = useRoute()
const serverId = route.params.id as string
const auth = await useAuth()

const props = defineProps<{
	server: ModrinthServer
	backupInProgress?: BackupInProgressReason
}>()

useHead({
	title: `Options - ${props.server.general?.name ?? 'Server'} - Modrinth`,
})

const ownerId = computed(() => props.server.general?.owner_id ?? 'Ghost')
const isOwner = computed(() => (auth.value?.user as User | null)?.id === ownerId.value)
const isAdmin = computed(() => isUserAdmin(auth.value?.user))

const navLinks = computed(() => [
	{ icon: SettingsIcon, label: 'General', href: `/servers/manage/${serverId}/options` },
	{ icon: WrenchIcon, label: 'Platform', href: `/servers/manage/${serverId}/options/loader` },
	{ icon: TextQuoteIcon, label: 'Startup', href: `/servers/manage/${serverId}/options/startup` },
	{ icon: VersionIcon, label: 'Network', href: `/servers/manage/${serverId}/options/network` },
	{ icon: ListIcon, label: 'Properties', href: `/servers/manage/${serverId}/options/properties` },
	{
		icon: UserIcon,
		label: 'Preferences',
		href: `/servers/manage/${serverId}/options/preferences`,
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
	{ icon: InfoIcon, label: 'Info', href: `/servers/manage/${serverId}/options/info` },
])
</script>
