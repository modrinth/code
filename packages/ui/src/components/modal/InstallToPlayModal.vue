<template>
	<NewModal ref="modal" header="Install to play" :closable="true">
		<div class="flex flex-col gap-4 max-w-[500px]">
			<Admonition type="info" header="Shared server instance">
				This server requires modded content to play. Accept to install the needed files from
				Modrinth.
			</Admonition>

			<div class="flex items-center gap-2 text-sm text-secondary" v-if="sharedBy?.name">
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
import type { Project } from '@modrinth/utils'
import { formatCategory } from '@modrinth/utils'
import { computed, ref } from 'vue'

import Admonition from '../base/Admonition.vue'
import Avatar from '../base/Avatar.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import NewModal from './NewModal.vue'

const props = defineProps<{
	project: Project
	sharedBy?: {
		name: string
		icon_url?: string
	}
	modCount?: number
}>()

const emit = defineEmits<{
	accept: []
	decline: []
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const loaderDisplay = computed(() => {
	const loader = props.project.loaders?.[0]
	if (!loader) return ''
	return formatCategory(loader)
})

function handleAccept() {
	// TODO: Implement accept logic
	emit('accept')
	modal.value?.hide()
}

function handleDecline() {
	emit('decline')
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
