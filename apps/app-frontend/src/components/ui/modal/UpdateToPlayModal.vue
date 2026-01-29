<template>
	<NewModal ref="modal" header="Update to play" :closable="true">
		<div class="flex flex-col gap-4 max-w-[500px]">
			<Admonition type="warning" header="Update required">
				An update is required to play {{ instance.name }}. Please update to the latest version to
				launch the game.
			</Admonition>
		</div>

		<template #actions>
			<div class="flex justify-between gap-2">
				<ButtonStyled color="red" type="transparent">
					<button @click="handleReport">
						<ReportIcon />
						Report
					</button>
				</ButtonStyled>
				<div class="flex gap-2">
					<ButtonStyled>
						<button @click="handleDecline">
							<XIcon />
							Decline
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button @click="handleUpdate">
							<CheckIcon />
							Accept
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, ReportIcon, XIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, NewModal } from '@modrinth/ui'
import { onMounted, ref } from 'vue'

import type { GameInstance } from '@/helpers/types'

const { instance, sharedBy } = defineProps<{
	instance: GameInstance
	sharedBy?: {
		name: string
		icon_url?: string
	}
}>()

async function checkUpdateAvailable(instance: GameInstance): Promise<boolean> {
	// inside instance, there is a linkedData field that has the project and version info
	// fetch the project id's versions, sort versions by date
	// compare current instance's linkedData version field to the project id's most recent version
	// console.log those two versions

	return false
}

onMounted(() => {
	checkUpdateAvailable(instance)
})

const modal = ref<InstanceType<typeof NewModal>>()

async function handleUpdate() {
	hide()
	try {
		// TODO, find update function, or delete prev instance and install new one?
		// await install(props.project.id, null, null, 'ProjectPageUpdateToPlayModal')
	} catch (error) {}
}

function handleReport() {
	// TODO handle what report button does
}

function handleDecline() {
	hide()
}

function show(e?: MouseEvent) {
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
