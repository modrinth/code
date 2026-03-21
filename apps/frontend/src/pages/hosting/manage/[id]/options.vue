<template>
	<div class="flex flex-col gap-4">
		<ServerSidebar :route="route" :nav-links="navLinks" />
	</div>
</template>
<script setup lang="ts">
import {
	CardIcon,
	ListIcon,
	ModrinthIcon,
	SettingsIcon,
	TextQuoteIcon,
	VersionIcon,
	WrenchIcon,
} from '@modrinth/assets'
import { injectModrinthServerContext } from '@modrinth/ui'
import { isAdmin as isUserAdmin, type User } from '@modrinth/utils'

import ServerSidebar from '~/components/ui/servers/ServerSidebar.vue'

const route = useRoute()
const serverId = route.params.id as string
const auth = await useAuth()

const { server } = injectModrinthServerContext()

useHead({
	title: `Options - ${server.value?.name ?? 'Server'} - Modrinth`,
})

const ownerId = computed(() => server.value?.owner_id ?? 'Ghost')
const isOwner = computed(() => (auth.value?.user as User | null)?.id === ownerId.value)
const isAdmin = computed(() => isUserAdmin(auth.value?.user))

const navLinks = computed(() => [
	{ icon: SettingsIcon, label: 'General', href: `/hosting/manage/${serverId}/options` },
	{ icon: WrenchIcon, label: 'Installation', href: `/hosting/manage/${serverId}/options/loader` },
	{ icon: VersionIcon, label: 'Network', href: `/hosting/manage/${serverId}/options/network` },
	{
		icon: ListIcon,
		label: 'Properties',
		href: `/hosting/manage/${serverId}/options/properties`,
		shown: server.value?.status !== 'installing',
	},
	{ icon: TextQuoteIcon, label: 'Advanced', href: `/hosting/manage/${serverId}/options/advanced` },
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
])
</script>
