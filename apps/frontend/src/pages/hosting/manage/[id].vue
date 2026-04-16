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
		:fetch-intercom-token="fetchIntercomToken"
		:intercom-app-id="config.public.intercomAppId as string"
		:navigate-to-billing="() => router.push('/settings/billing')"
		:navigate-to-servers="() => router.push('/hosting/manage')"
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
import { ServersManageRootLayout } from '@modrinth/ui'

import { reloadNuxtApp } from '#app'
import { products } from '~/generated/state.json'

const flags = useFeatureFlags()
const route = useNativeRoute()
const router = useRouter()
const config = useRuntimeConfig()
const serverId = route.params.id as string

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

async function fetchIntercomToken(): Promise<{ token: string }> {
	return $fetch('/api/intercom/messenger-jwt', {
		query: { server_id: serverId },
	})
}

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
