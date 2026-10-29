<template>
	<ServersManageRootLayout
		:server-id="serverId"
		:reload-page="() => reloadNuxtApp({ path: route.path })"
		:resolve-viewer="resolveViewer"
		:show-copy-id-action="flags.developerMode"
		:show-advanced-debug-info="flags.advancedDebugInfo"
		:force-server-play-migration="flags.debugServerPlayMigration"
		:stripe-publishable-key="config.public.stripePublishableKey as string"
		:site-url="config.public.siteUrl as string"
		:products="products"
		:auth-user="authUser"
		:navigate-to-billing="() => router.push('/settings/billing')"
		:navigate-to-admin-billing="(ownerId: string) => router.push(`/admin/billing/${ownerId}`)"
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
		@server-play-migration-tagged="serverSharingUpdate.show"
	>
		<template #default="{ onReinstall, onReinstallFailed }">
			<NuxtPage :route="route" @reinstall="onReinstall" @reinstall-failed="onReinstallFailed" />
			<ServerSharingUpdateModal
				ref="serverSharingUpdateModal"
				:site-url="config.public.siteUrl as string"
				:description="formatMessage(messages.sharingUpdateDescription)"
				video-src="https://cdn.modrinth.com/media/server-play-demo.webm"
				poster-src="https://cdn.modrinth.com/media/server-play-demo.webp"
			/>
		</template>
	</ServersManageRootLayout>
</template>

<script setup lang="ts">
import {
	defineMessages,
	injectModrinthClient,
	ServerSharingUpdateModal,
	ServersManageRootLayout,
	useVIntl,
} from '@modrinth/ui'
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

const { formatMessage } = useVIntl()
const serverSharingUpdateModal = useTemplateRef<InstanceType<typeof ServerSharingUpdateModal>>(
	'serverSharingUpdateModal',
)
const serverSharingUpdate = useServerSharingUpdate(serverSharingUpdateModal, serverId)

const messages = defineMessages({
	sharingUpdateDescription: {
		id: 'servers.sharing-update.web-description',
		defaultMessage:
			'You can now share your server directly with friends. When they accept in the Modrinth App, we’ll create an instance for them that is managed by your server.',
	},
})

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
