<template>
	<NewModal ref="modal" header="Install to play" :closable="true">
		<div class="flex flex-col gap-4 max-w-[500px]">
			<Admonition type="info" header="Shared server instance">
				This server requires modded content to play. Accept to install the needed files from
				Modrinth.
			</Admonition>

			<div v-if="sharedBy?.name" class="flex items-center gap-2 text-sm text-secondary">
				<Avatar
					v-if="sharedBy?.icon_url"
					:src="sharedBy.icon_url"
					:alt="sharedBy.name"
					size="24px"
				/>
				<span>
					<span class="font-semibold text-contrast">{{ sharedBy.name }}</span>
					shared this instance with you today.
				</span>
			</div>

			<div class="flex flex-col gap-2">
				<span class="text-sm font-semibold text-secondary">Shared instance</span>
				<div class="flex items-center gap-3 rounded-xl bg-surface-4 p-3">
					<Avatar :src="project.icon_url" :alt="project.title" size="48px" />
					<div class="flex flex-col gap-0.5">
						<span class="font-semibold text-contrast">{{ project.title }}</span>
						<span class="text-sm text-secondary">
							{{ loaderDisplay }} {{ project.game_versions?.[0] }}
							<template v-if="modCount"> · {{ modCount }} mods </template>
						</span>
					</div>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button @click="handleDecline">
						<XIcon />
						Decline
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="handleAccept">
						<CheckIcon />
						Accept
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { Admonition, Avatar, ButtonStyled, NewModal } from '@modrinth/ui'
import { formatCategory } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { get_version } from '@/helpers/cache.js'
import { install } from '@/store/install.js'
import type { Labrinth } from '@modrinth/api-client'

const props = defineProps<{
	project: Labrinth.Projects.v2.Project
	sharedBy?: {
		name: string
		icon_url?: string
	}
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const loaderDisplay = computed(() => {
	const loader = props.project.loaders?.[0]
	if (!loader) return ''
	return formatCategory(loader)
})

// Fetch the most recent version to get mod count from dependencies
const latestVersionId = computed(() => props.project.versions?.[0] ?? null)
const { data: latestVersion } = useQuery({
	queryKey: computed(() => ['version', latestVersionId.value]),
	queryFn: () => get_version(latestVersionId.value, 'must_revalidate'),
	enabled: computed(() => !!latestVersionId.value),
})
const modCount = computed(() => latestVersion.value?.dependencies?.length)

async function handleAccept() {
	modal.value?.hide()
	try {
		await install(props.project.id, null, null, 'ProjectPageInstallToPlayModal')
	} catch (error) {}
}

function handleDecline() {
	modal.value?.hide()
}

function show(e?: MouseEvent) {
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
