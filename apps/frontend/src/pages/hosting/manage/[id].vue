<template>
	<ServersManageRootLayout
		:server-id="serverId"
		:reload-page="() => reloadNuxtApp({ path: route.path })"
		:resolve-viewer="resolveViewer"
		:show-copy-id-action="flags.developerMode"
		:show-advanced-debug-info="flags.advancedDebugInfo"
		:stripe-publishable-key="config.public.stripePublishableKey as string"
		:site-url="config.public.siteUrl as string"
		:products="products"
		:auth-user="authUser"
		:navigate-to-billing="() => router.push('/settings/billing')"
		:navigate-to-admin-billing="(ownerId: string) => router.push(`/admin/billing/${ownerId}`)"
		:navigate-to-servers="() => router.push('/hosting/manage')"
		constrain-width
		:browse-modpacks="browseModpacks"
		:browse-content="browseContent"
	>
		<template #default="{ onReinstall, onReinstallFailed }">
			<NuxtPage
				:route="panelRoute"
				@reinstall="onReinstall"
				@reinstall-failed="onReinstallFailed"
			/>
		</template>
	</ServersManageRootLayout>
</template>

<script setup lang="ts">
import { injectModrinthClient, ServersManageRootLayout } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

import { reloadNuxtApp } from '#app'
import { prefetchServerInstallContent } from '~/composables/use-server-install-content'
import { products } from '~/generated/state.json'

const flags = useFeatureFlags()
const route = useNativeRoute()
const router = useRouter()
const config = useRuntimeConfig()
const serverId = route.params.id as string

const client = injectModrinthClient()
const queryClient = useQueryClient()

const panelPath = `/hosting/manage/${serverId}`
const panelRoute = shallowRef(router.currentRoute.value)
watch(
	() => router.currentRoute.value,
	(current) => {
		if (current.path === panelPath || current.path.startsWith(`${panelPath}/`)) {
			panelRoute.value = current
		}
	},
)

function browseModpacks({
	serverId: sid,
	worldId: wid,
	from,
}: {
	serverId: string
	worldId: string | null
	from: 'reset-server' | 'onboarding'
}) {
	void prefetchServerInstallContent(queryClient, client, sid, wid)
	navigateTo({ path: '/discover/modpacks', query: { sid, from, wid: wid ?? undefined } })
}

function browseContent({
	serverId: sid,
	worldId: wid,
	type,
}: {
	serverId: string
	worldId: string | null
	type: 'mod' | 'plugin' | 'datapack'
}) {
	void prefetchServerInstallContent(queryClient, client, sid, wid)
	navigateTo({ path: `/discover/${type}s`, query: { sid, wid: wid ?? undefined } })
}

if (serverId) {
	try {
		await queryClient.ensureQueryData({
			queryKey: ['servers', 'detail', serverId],
			queryFn: () => client.archon.servers_v0.get(serverId)!,
			staleTime: 30_000,
		})
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	}
}

const auth = (await useAuth()) as unknown as {
	value: { user: { id: string; username: string; email: string; created: string } }
}

const authUser = auth.value?.user
	? {
			id: auth.value.user.id,
			username: auth.value.user.username,
			email: auth.value.user.email,
			created: auth.value.user.created,
		}
	: undefined

async function resolveViewer(): Promise<{ userId: string | null; userRole: string | null }> {
	return {
		userId: auth.value?.user?.id ?? null,
		userRole: (auth.value?.user as any)?.role ?? null,
	}
}

definePageMeta({
	middleware: 'auth',
})

useHead({
	script: [
		{
			src: 'https://tally.so/widgets/embed.js',
			defer: true,
		},
	],
})
</script>
