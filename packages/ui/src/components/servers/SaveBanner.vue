<template>
	<Teleport v-if="!saveBanner || saveBanner.target.value" :to="saveBanner?.target.value ?? 'body'">
		<FloatingActionBar :shown="props.isVisible" :inline="!!saveBanner">
			<p class="m-0 font-semibold text-sm md:text-base">You have unsaved changes.</p>
			<div class="ml-auto flex gap-2">
				<Button type="quiet" :disabled="props.isUpdating" @click="props.reset"
					><HistoryIcon /> Reset</Button
				>
				<Button
					:type="props.restart ? 'base' : 'colored'"
					:color="props.restart ? undefined : 'brand'"
					:disabled="props.isUpdating"
					@click="props.save"
				>
					<SpinnerIcon v-if="props.isUpdating" class="animate-spin" />
					<SaveIcon v-else />
					{{ props.isUpdating ? 'Saving...' : 'Save' }}
				</Button>
				<Button
					v-if="props.restart"
					type="colored"
					color="brand"
					:disabled="props.isUpdating || isTransitioning"
					@click="saveAndPower"
				>
					<SpinnerIcon v-if="props.isUpdating || isTransitioning" class="animate-spin" />
					{{ powerButtonLabel }}
				</Button>
			</div>
		</FloatingActionBar>
	</Teleport>
</template>

<script setup lang="ts">
import { HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, watch } from 'vue'

import { Button } from '#ui/components/base/buttons'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import { injectServerSettings } from '#ui/layouts/shared/server-settings/providers/server-settings'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

const props = defineProps<{
	isUpdating: boolean
	restart?: boolean
	save: () => void | Promise<void>
	reset: () => void
	isVisible: boolean
	serverId: string
}>()

const saveBanner = injectServerSettings(null)?.saveBanner

watch(
	() => props.isVisible,
	(shown) => {
		if (saveBanner) saveBanner.shown.value = shown
	},
	{ immediate: true },
)

onBeforeUnmount(() => {
	if (saveBanner) saveBanner.shown.value = false
})

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
