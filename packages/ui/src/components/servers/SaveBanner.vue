<template>
	<Teleport v-if="!saveBanner || saveBanner.target.value" :to="saveBanner?.target.value ?? 'body'">
		<FloatingActionBar :shown="props.isVisible" :inline="!!saveBanner">
			<p class="m-0 font-semibold text-sm md:text-base">You have unsaved changes.</p>
			<div class="ml-auto flex gap-2">
				<Button type="quiet" :disabled="props.isUpdating || saved" @click="handleReset"
					><HistoryIcon /> Reset</Button
				>
				<span v-tooltip="saveDisabledTooltip" class="flex">
					<Button
						:type="props.restart ? 'base' : 'colored'"
						:color="props.restart ? undefined : 'brand'"
						:disabled="props.isUpdating || props.hasErrors || saved"
						@click="handleSave"
					>
						<SpinnerIcon v-if="props.isUpdating" class="animate-spin" />
						<CheckIcon v-else-if="saved" />
						<SaveIcon v-else />
						{{ saveButtonLabel }}
					</Button>
				</span>
				<span v-if="props.restart" v-tooltip="saveDisabledTooltip" class="flex">
					<Button
						type="colored"
						color="brand"
						:disabled="props.isUpdating || isTransitioning || props.hasErrors || saved"
						@click="saveAndPower"
					>
						<SpinnerIcon v-if="props.isUpdating || isTransitioning" class="animate-spin" />
						{{ powerButtonLabel }}
					</Button>
				</span>
			</div>
		</FloatingActionBar>
	</Teleport>
</template>

<script setup lang="ts">
import { CheckIcon, HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { Button } from '#ui/components/base/buttons'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import { defineMessage, useVIntl } from '#ui/composables/i18n'
import { injectServerSettings } from '#ui/layouts/shared/server-settings/providers/server-settings'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

const props = defineProps<{
	isUpdating: boolean
	hasErrors?: boolean
	restart?: boolean
	save: () => void | boolean | Promise<void | boolean>
	reset: () => void
	isVisible: boolean
	serverId: string
}>()

const { formatMessage } = useVIntl()
const resolveErrorsMessage = defineMessage({
	id: 'hosting.save-banner.resolve-errors',
	defaultMessage: 'Please resolve the errors in your changes before saving.',
})
const saveDisabledTooltip = computed(() =>
	props.hasErrors ? formatMessage(resolveErrorsMessage) : undefined,
)

const savedMessage = defineMessage({
	id: 'hosting.save-banner.saved',
	defaultMessage: 'Saved',
})

const saveBanner = injectServerSettings(null)?.saveBanner

const saveRequested = ref(false)
const saved = ref(false)

const saveButtonLabel = computed(() => {
	if (props.isUpdating) return 'Saving...'
	if (saved.value) return formatMessage(savedMessage)
	return 'Save'
})

function handleSave() {
	saveRequested.value = true
	return props.save()
}

function handleReset() {
	saveRequested.value = false
	props.reset()
}

watch(
	() => props.isVisible,
	(shown) => {
		if (saveBanner) saveBanner.shown.value = shown
		saved.value = !shown && saveRequested.value
		saveRequested.value = false
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
	if (props.hasErrors) return
	try {
		if ((await handleSave()) === false) return
	} catch {
		return
	}
	await client.archon.servers_v0.power(props.serverId, isStopped.value ? 'Start' : 'Restart')
}
</script>
