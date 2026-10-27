<script setup lang="ts">
import { DownloadIcon, EyeIcon, MoreVerticalIcon } from '@modrinth/assets'
import {
	Avatar,
	defineMessages,
	formatLoader,
	injectAuth,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import {
	cachedSharedInstanceLibraryQueryOptions,
	type UninstalledSharedInstance,
	useHiddenInstances,
} from '@/components/ui/library/use-uninstalled-shared-instances'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import { getInstanceIconUrl } from '@/helpers/instance'

const { formatMessage } = useVIntl()
const auth = injectAuth()
const router = useRouter()
const settingsModal = injectAppSettingsModal()

const messages = defineMessages({
	title: {
		id: 'app.behavior-settings.hidden-instances.title',
		defaultMessage: 'Hidden instances',
	},
	description: {
		id: 'app.behavior-settings.hidden-instances.description',
		defaultMessage:
			"Instances you've hidden from your library on this device. Hidden instances aren't synced with your Modrinth account.",
	},
	empty: {
		id: 'app.behavior-settings.hidden-instances.empty',
		defaultMessage: "You haven't hidden any instances.",
	},
	moreOptions: {
		id: 'app.behavior-settings.hidden-instances.more-options',
		defaultMessage: 'More options',
	},
	showInLibrary: {
		id: 'app.behavior-settings.hidden-instances.show-in-library',
		defaultMessage: 'Show in library',
	},
	install: {
		id: 'app.library.instance.install',
		defaultMessage: 'Install',
	},
})

const userId = computed(() => auth.user.value?.id ?? null)
const cachedQuery = useQuery(cachedSharedInstanceLibraryQueryOptions(userId))
const { hiddenIds, setHidden } = useHiddenInstances()

const hiddenInstances = computed(() => {
	const cachedById = new Map(
		(cachedQuery.data.value ?? []).map((instance) => [instance.id, instance]),
	)
	return [...hiddenIds.value].flatMap((instanceId) => {
		const instance = cachedById.get(instanceId)
		return instance ? [instance] : []
	})
})

const install = async (instance: UninstalledSharedInstance) => {
	await setHidden(instance.id, false)
	if (!settingsModal.close()) return
	await router.push({ path: '/', query: { installSharedInstance: instance.shared_instance_id } })
}

const getOptions = (instance: UninstalledSharedInstance) => [
	{
		id: 'show-in-library',
		label: formatMessage(messages.showInLibrary),
		icon: EyeIcon,
		action: () => void setHidden(instance.id, false),
	},
	{
		id: 'install',
		label: formatMessage(messages.install),
		icon: DownloadIcon,
		tone: 'brand' as const,
		action: () => void install(instance),
	},
]
</script>

<template>
	<section v-if="auth.user.value" class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.title) }}
		</h2>
		<p class="m-0 mt-1 text-secondary">{{ formatMessage(messages.description) }}</p>
		<div v-if="hiddenInstances.length > 0" class="mt-4 flex flex-col gap-2">
			<div
				v-for="instance in hiddenInstances"
				:key="instance.id"
				class="flex items-center gap-3 rounded-2xl bg-surface-2 p-3"
			>
				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					cache-icon
					:tint-by="instance.id"
					:alt="instance.name"
					size="56px"
					no-shadow
					class="!rounded-2xl"
					pad-transparent-corners
				/>
				<div class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
					<span class="truncate text-sm font-medium text-secondary">
						{{ formatLoader(formatMessage, instance.loader) }} {{ instance.game_version }}
					</span>
				</div>
				<TeleportOverflowMenu
					type="quiet"
					:label="formatMessage(messages.moreOptions)"
					:options="getOptions(instance)"
				>
					<MoreVerticalIcon class="size-5" />
				</TeleportOverflowMenu>
			</div>
		</div>
		<p v-else class="m-0 mt-4 text-secondary">{{ formatMessage(messages.empty) }}</p>
	</section>
</template>
