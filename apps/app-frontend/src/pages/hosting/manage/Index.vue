<template>
	<div
		class="h-full w-full pt-6"
		:class="isContainedServerRoute ? 'box-border min-h-0 overflow-hidden' : ''"
	>
		<ServersManageRootLayout
			:server-id="serverId"
			:layout-mode="isContainedServerRoute ? 'contained' : 'page'"
			:reload-page="() => router.go(0)"
			:resolve-viewer="resolveViewer"
			:show-copy-id-action="themeStore.devMode"
			:auth-user="authUser"
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
import {
	commonMessages,
	injectAuth,
	injectModrinthClient,
	ServersManageRootLayout,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { get_user } from '@/helpers/cache'
import { get as getCreds } from '@/helpers/mr_auth'
import {
	provideBreadcrumbParent,
	useBreadcrumb,
} from '@/providers/breadcrumbs'
import { useTheming } from '@/store/theme'

const route = useRoute()
const router = useRouter()
const auth = injectAuth()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const themeStore = useTheming()
const { formatMessage } = useVIntl()

const isContainedServerRoute = computed(() => route.name === 'ServerManageOverview')

const serverId = computed(() => {
	const rawId = route.params.id
	return Array.isArray(rawId) ? (rawId[0] ?? '') : (rawId ?? '')
})

const { data: serverData } = useQuery({
	queryKey: computed(() => ['servers', 'detail', serverId.value]),
	queryFn: () => null as unknown as Archon.Servers.v0.Server,
	enabled: false,
})

const breadcrumbServerId = ref(serverId.value)
const breadcrumbLabel = ref(formatMessage(commonMessages.loadingLabel))
watch(
	serverId,
	(value) => {
		if (!route.path.startsWith('/hosting/manage/') || route.name === 'Servers') return
		breadcrumbServerId.value = value
		breadcrumbLabel.value = formatMessage(commonMessages.loadingLabel)
	},
	{ flush: 'sync' },
)
watch(
	serverData,
	(server) => {
		if (!route.path.startsWith('/hosting/manage/') || !server?.name) return
		breadcrumbLabel.value = server.name
	},
	{ immediate: true },
)

const serverBreadcrumb = useBreadcrumb({
	slot: 'server',
	id: () => `server:${breadcrumbServerId.value}`,
	label: breadcrumbLabel,
	to: () => `/hosting/manage/${encodeURIComponent(breadcrumbServerId.value)}`,
})
provideBreadcrumbParent(serverBreadcrumb)

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
