<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, EyeIcon } from '@modrinth/assets'
import { Avatar, Button, defineMessages, injectAuth, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import {
	cachedSharedInstanceLibraryQueryOptions,
	useHiddenSharedInstances,
} from '@/components/ui/library/use-uninstalled-shared-instances'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import { get_user_many } from '@/helpers/cache.js'
import { getInstanceIconUrl } from '@/helpers/instance'

const { formatMessage } = useVIntl()
const auth = injectAuth()
const router = useRouter()
const settingsModal = injectAppSettingsModal()

const messages = defineMessages({
	title: {
		id: 'app.behavior-settings.hidden-shared-instances.title',
		defaultMessage: 'Hidden shared instances',
	},
	description: {
		id: 'app.behavior-settings.hidden-shared-instances.description',
		defaultMessage:
			"Shared instances you've hidden from your library on this device. Hidden instances aren't synced, and you're still a member of them.",
	},
	empty: {
		id: 'app.behavior-settings.hidden-shared-instances.empty',
		defaultMessage:
			"You haven't hidden any shared instances. To hide one, click the × on a shared instance in your library that isn't installed.",
	},
	sharedBy: {
		id: 'app.library.instance.shared-by-prefix',
		defaultMessage: 'Shared by',
	},
	sharedByYou: {
		id: 'app.library.instance.shared-by-you',
		defaultMessage: 'Shared by you',
	},
	showInLibrary: {
		id: 'app.behavior-settings.hidden-shared-instances.show-in-library',
		defaultMessage: 'Show in library',
	},
	install: {
		id: 'app.library.instance.install',
		defaultMessage: 'Install',
	},
})

const userId = computed(() => auth.user.value?.id ?? null)
const cachedQuery = useQuery(cachedSharedInstanceLibraryQueryOptions(userId))
const { hiddenIds, setHidden } = useHiddenSharedInstances()

const hiddenInstances = computed(() => {
	const cachedById = new Map(
		(cachedQuery.data.value ?? []).map((instance) => [instance.shared_instance_id, instance]),
	)
	return [...hiddenIds.value].flatMap((sharedInstanceId) => {
		const instance = cachedById.get(sharedInstanceId)
		return instance ? [instance] : []
	})
})

const ownerIds = computed(() =>
	[
		...new Set(
			hiddenInstances.value.flatMap((instance) =>
				instance.role !== 'owner' && instance.manager_id ? [instance.manager_id] : [],
			),
		),
	].sort(),
)

const ownersQuery = useQuery({
	queryKey: computed(() => ['shared-instances', 'owners', ...ownerIds.value]),
	queryFn: async () => (await get_user_many(ownerIds.value)) as Labrinth.Users.v2.User[],
	enabled: computed(() => ownerIds.value.length > 0),
	staleTime: 5 * 60_000,
})

const ownersById = computed(
	() => new Map((ownersQuery.data.value ?? []).map((user) => [user.id, user])),
)

const install = async (sharedInstanceId: string) => {
	await setHidden(sharedInstanceId, false)
	if (!settingsModal.close()) return
	await router.push({ path: '/', query: { installSharedInstance: sharedInstanceId } })
}
</script>

<template>
	<section v-if="auth.user.value" class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.title) }}
		</h2>
		<p class="m-0 mt-1 text-secondary">{{ formatMessage(messages.description) }}</p>
		<div v-if="hiddenInstances.length > 0" class="mt-4 flex flex-col gap-3">
			<div
				v-for="instance in hiddenInstances"
				:key="instance.shared_instance_id"
				class="flex items-center gap-3"
			>
				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					cache-icon
					:tint-by="instance.id"
					size="48px"
					alt=""
					no-shadow
				/>
				<div class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
					<span class="truncate text-sm capitalize text-secondary">
						{{ instance.loader }} {{ instance.game_version }}
					</span>
					<span class="flex items-center gap-1 text-sm text-secondary">
						<template v-if="instance.role === 'owner'">
							{{ formatMessage(messages.sharedByYou) }}
						</template>
						<template v-else-if="instance.manager_id && ownersById.get(instance.manager_id)">
							{{ formatMessage(messages.sharedBy) }}
							<Avatar
								:src="ownersById.get(instance.manager_id)!.avatar_url"
								cache-icon
								:tint-by="ownersById.get(instance.manager_id)!.username"
								size="16px"
								alt=""
								circle
								no-shadow
							/>
							{{ ownersById.get(instance.manager_id)!.username }}
						</template>
					</span>
				</div>
				<Button @click="setHidden(instance.shared_instance_id, false)">
					<EyeIcon />
					{{ formatMessage(messages.showInLibrary) }}
				</Button>
				<Button type="colored" color="brand" @click="install(instance.shared_instance_id)">
					<DownloadIcon />
					{{ formatMessage(messages.install) }}
				</Button>
			</div>
		</div>
		<p v-else class="m-0 mt-4 text-secondary">{{ formatMessage(messages.empty) }}</p>
	</section>
</template>
