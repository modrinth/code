<template>
	<div class="h-full w-full pt-6">
		<ServersManageRootLayout
			:server-id="serverId"
			:reload-page="() => router.go(0)"
			:resolve-viewer="resolveViewer"
			:show-copy-id-action="themeStore.devMode"
			:auth-user="authUser"
			:fetch-intercom-token="fetchIntercomToken"
			:navigate-to-billing="() => openUrl('https://modrinth.com/settings/billing')"
			:navigate-to-servers="() => router.push('/hosting/manage')"
			:browse-modpacks="
				({ serverId: sid, worldId: wid, from }) => {
					router.push({
						path: '/browse/modpack',
						query: { sid, wid: wid ?? undefined, from },
					})
				}
			"
			:browse-content="
				({ serverId: sid, worldId: wid, type }) => {
					router.push({
						path: `/browse/${type}`,
						query: { sid, wid: wid ?? undefined },
					})
				}
			"
		>
			<template #default="{ onReinstall, onReinstallFailed }">
				<RouterView v-slot="{ Component }">
					<template v-if="Component">
						<Suspense>
							<component
								:is="Component"
								@reinstall="onReinstall"
								@reinstall-failed="onReinstallFailed"
							/>
						</Suspense>
					</template>
				</RouterView>
			</template>
		</ServersManageRootLayout>
	</div>
</template>

<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import { injectAuth, injectModrinthClient, ServersManageRootLayout } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { config } from '@/config'
import { get_user } from '@/helpers/cache'
import { get as getCreds } from '@/helpers/mr_auth'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useTheming } from '@/store/theme'

const route = useRoute()
const router = useRouter()
const auth = injectAuth()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const themeStore = useTheming()
const breadcrumbs = useBreadcrumbs()

const serverId = computed(() => {
	const rawId = route.params.id
	return Array.isArray(rawId) ? rawId[0] : (rawId ?? '')
})

if (serverId.value) {
	try {
		await queryClient.ensureQueryData({
			queryKey: ['servers', 'detail', serverId.value],
			queryFn: () => client.archon.servers_v0.get(serverId.value)!,
			staleTime: 30_000,
		})
	} catch {
		// Let mounted layouts' useQuery surface errors; do not fail route setup.
	}
}

const { data: serverData } = useQuery({
	queryKey: computed(() => ['servers', 'detail', serverId.value]),
	queryFn: () => null as unknown as Archon.Servers.v0.Server,
	enabled: false,
})

watch(
	serverData,
	(server) => {
		if (server?.name) {
			breadcrumbs.setName('Server', server.name)
			breadcrumbs.setContext({
				name: server.name,
				link: `/hosting/manage/${serverId.value}/content`,
			})
		}
	},
	{ immediate: true },
)

watch(
	() => auth.user.value,
	(user, previousUser) => {
		if (user || !previousUser) return
		if (route.path === '/hosting/manage' || route.path === '/hosting/manage/') return
		void router.replace('/hosting/manage')
	},
)

const authUser = computed(() => {
	const user = auth.user.value
	if (!user?.id) return undefined
	return {
		id: user.id,
		username: user.username,
		email: user.email ?? '',
		created: user.created,
	}
})

async function fetchIntercomToken(): Promise<{ token: string }> {
	const credentials = await getCreds()
	if (!credentials?.session) {
		throw new Error('Not authenticated')
	}
	const response = await tauriFetch(
		`${config.siteUrl}/api/intercom/messenger-jwt?server_id=${encodeURIComponent(serverId.value)}`,
		{
			method: 'GET',
			headers: {
				Authorization: `Bearer ${credentials.session}`,
			},
		},
	)
	if (!response.ok) {
		throw new Error(`Failed to fetch Intercom token: ${response.status}`)
	}
	return (await response.json()) as { token: string }
}

async function resolveViewer(): Promise<{ userId: string | null; userRole: string | null }> {
	const credentials = await getCreds().catch(() => null)
	if (!credentials?.user_id) {
		return { userId: null, userRole: null }
	}

	const user = await get_user(credentials.user_id, 'bypass').catch(() => null)
	const typedUser = user as Labrinth.Users.v2.User | null
	return {
		userId: credentials.user_id,
		userRole: typedUser?.role ?? null,
	}
}
</script>
