<template>
	<ServersManageRootLayout
		:server-id="serverId"
		:reload-page="() => router.go(0)"
		:resolve-viewer="resolveViewer"
		:show-uptime="false"
		:show-copy-id-action="themeStore.devMode"
		:navigate-to-billing="() => router.push('/settings/billing')"
		:navigate-to-servers="() => router.push('/hosting/manage')"
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
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { LoadingIndicator, ServersManageRootLayout } from '@modrinth/ui'
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { get_user } from '@/helpers/cache'
import { get as getCreds } from '@/helpers/mr_auth'
import { useTheming } from '@/store/theme'

const route = useRoute()
const router = useRouter()
const themeStore = useTheming()

const serverId = computed(() => {
	const rawId = route.params.id
	return Array.isArray(rawId) ? rawId[0] : (rawId ?? '')
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
