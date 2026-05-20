<template>
	<Teleport to="body">
		<FloatingActionBar :shown="props.isVisible">
			<p class="m-0 font-semibold text-sm md:text-base">You have unsaved changes.</p>
			<div class="ml-auto flex gap-2">
				<ButtonStyled type="transparent">
					<button :disabled="props.isUpdating" @click="props.reset"><HistoryIcon /> Reset</button>
				</ButtonStyled>
				<ButtonStyled :color="props.restart ? 'standard' : 'brand'">
					<button :disabled="props.isUpdating" @click="props.save">
						<SpinnerIcon v-if="props.isUpdating" class="animate-spin" />
						<SaveIcon v-else />
						{{ props.isUpdating ? 'Saving...' : 'Save' }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="props.restart" color="brand">
					<button :disabled="props.isUpdating || isTransitioning" @click="saveAndPower">
						<SpinnerIcon v-if="props.isUpdating || isTransitioning" class="animate-spin" />
						{{ powerButtonLabel }}
					</button>
				</ButtonStyled>
			</div>
		</FloatingActionBar>
	</Teleport>
</template>

<script setup lang="ts">
import { HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { computed } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

const props = defineProps<{
	isUpdating: boolean
	restart?: boolean
	save: () => void | Promise<void>
	reset: () => void
	isVisible: boolean
	serverId: string
}>()

const client = injectModrinthClient()

const { powerState } = injectModrinthServerContext()

const isStopped = computed(() => powerState.value === 'stopped' || powerState.value === 'crashed')

const isTransitioning = computed(
	() => powerState.value === 'starting' || powerState.value === 'stopping',
)

const powerButtonLabel = computed(() => {
	if (props.isUpdating) return 'Saving...'
	if (isTransitioning.value) return isStopped.value ? 'Starting...' : 'Restarting...'
	return isStopped.value ? 'Save & start' : 'Save & restart'
})

const saveAndPower = async () => {
	try {
		await props.save()
	} catch {
		return
	}
	await client.archon.servers_v0.power(props.serverId, isStopped.value ? 'Start' : 'Restart')
}
</script>
