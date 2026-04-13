<template>
	<div class="h-full w-full py-6">
		<ServersManageRootLayout
			:server-id="serverId"
			:reload-page="() => router.go(0)"
			:resolve-viewer="resolveViewer"
			:show-copy-id-action="themeStore.devMode"
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
							<template #fallback>
								<LoadingIndicator />
							</template>
						</Suspense>
					</template>
				</RouterView>
			</template>
		</ServersManageRootLayout>
	</div>
</template>

<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import { injectAuth, LoadingIndicator, ServersManageRootLayout } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { get_user } from '@/helpers/cache'
import { get as getCreds } from '@/helpers/mr_auth'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useTheming } from '@/store/theme'

const route = useRoute()
const router = useRouter()
const auth = injectAuth()
const themeStore = useTheming()
const breadcrumbs = useBreadcrumbs()

const serverId = computed(() => {
	const rawId = route.params.id
	return Array.isArray(rawId) ? rawId[0] : (rawId ?? '')
})

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
