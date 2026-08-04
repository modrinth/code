<template>
	<ServersManageRootLayout
		:server-id="serverId"
		:reload-page="() => reloadNuxtApp({ path: route.path })"
		:resolve-viewer="resolveViewer"
		:show-advanced-debug-info="flags.advancedDebugInfo"
		:stripe-publishable-key="config.public.stripePublishableKey as string"
		:site-url="config.public.siteUrl as string"
		:products="products"
		:auth-user="authUser"
		:navigate-to-billing="() => router.push('/settings/billing')"
		:navigate-to-servers="() => router.push('/hosting/manage')"
		constrain-width
		:browse-modpacks="
			({ serverId: sid, worldId: wid, from }) => {
				navigateTo({
					path: '/discover/modpacks',
					query: { sid, from, wid: wid ?? undefined },
				})
			}
		"
		:browse-content="
			({ serverId: sid, worldId: wid, type }) => {
				navigateTo({
					path: `/discover/${type}s`,
					query: { sid, wid: wid ?? undefined },
				})
			}
		"
	>
		<template #default="{ onReinstall, onReinstallFailed }">
			<NuxtPage :route="route" @reinstall="onReinstall" @reinstall-failed="onReinstallFailed" />
		</template>
	</ServersManageRootLayout>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { injectModrinthClient, ServersManageRootLayout } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

import { reloadNuxtApp } from '#app'
import { products } from '~/generated/state.json'

const flags = useFeatureFlags()
const route = useNativeRoute()
const router = useRouter()
const config = useRuntimeConfig()
const serverId = route.params.id as string

const client = injectModrinthClient()
const queryClient = useQueryClient()

if (serverId) {
	const serverDetailPromise = queryClient.ensureQueryData({
		queryKey: ['servers', 'detail', serverId],
		queryFn: () => client.archon.servers_v0.get(serverId)!,
		staleTime: 30_000,
	})
	const serverFullPromise = queryClient.ensureQueryData({
		queryKey: ['servers', 'v1', 'detail', serverId],
		queryFn: () => client.archon.servers_v1.get(serverId),
		staleTime: 30_000,
	})
	const [, serverFullResult] = await Promise.allSettled([serverDetailPromise, serverFullPromise])

	if (serverFullResult.status === 'fulfilled') {
		const worldId = resolveWorldId(route.params.instance_id, serverFullResult.value)
		if (worldId) {
			await Promise.allSettled([
				queryClient.ensureQueryData({
					queryKey: ['backups', 'queue', serverId, worldId],
					queryFn: () => client.archon.backups_queue_v1.list(serverId, worldId),
					staleTime: 30_000,
				}),
			])
		}
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

function resolveWorldId(
	routeInstanceId: string | string[] | undefined,
	serverFull: Archon.Servers.v1.ServerFull,
) {
	const instanceId = getRouteParam(routeInstanceId)
	if (instanceId) return instanceId
	const activeWorld = serverFull.worlds.find((world) => world.is_active)
	return activeWorld?.id ?? serverFull.worlds[0]?.id ?? null
}

function getRouteParam(param: string | string[] | undefined): string | null {
	if (Array.isArray(param)) return param[0] ?? null
	return param ?? null
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
