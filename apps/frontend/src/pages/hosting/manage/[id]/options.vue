<template>
	<div class="flex flex-col gap-4">
		<ServerSidebar :route="route" :nav-links="navLinks" />
	</div>
</template>
<script setup lang="ts">
import {
	getServerSettingsNavLinks,
	injectModrinthServerContext,
	provideServerSettings,
	type ServerSettingsBrowseModpacksArgs,
} from '@modrinth/ui'
import { isAdmin as isUserAdmin, type User } from '@modrinth/utils'
import { computed, ref } from 'vue'

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

const currentUserId = computed(() => (auth.value?.user as User | null)?.id ?? null)
const currentUserRole = computed(() => auth.value?.user?.role ?? null)

provideServerSettings({
	isApp: ref(false),
	currentUserId,
	currentUserRole,
	browseModpacks: ({ serverId, worldId, from }: ServerSettingsBrowseModpacksArgs) => {
		navigateTo({
			path: '/discover/modpacks',
			query: { sid: serverId, from, wid: worldId ?? undefined },
		})
	},
})

const navLinks = computed(() =>
	getServerSettingsNavLinks({
		serverId,
		ownerId: ownerId.value,
		serverStatus: server.value?.status,
		isOwner: isOwner.value,
		isAdmin: isAdmin.value,
	}),
)
</script>
