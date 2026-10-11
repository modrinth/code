<template>
	<!-- Same panel as the "Existing instance" list of ContentInstallModal, edge to edge in the modal. -->
	<div
		class="-mx-6 -mt-6 flex flex-col gap-3 overflow-y-auto bg-surface-2 py-4"
		style="height: 400px"
	>
		<div class="px-6">
			<Input
				v-model="searchFilter"
				:icon="SearchIcon"
				:placeholder="formatMessage(messages.searchPlaceholder)"
			/>
		</div>
		<div v-if="loading" class="flex items-center justify-center py-12">
			<LoadingIndicator />
		</div>
		<div
			v-else-if="filteredInstances.length === 0"
			class="flex items-center justify-center py-12 text-secondary"
		>
			{{ formatMessage(messages.noInstances) }}
		</div>
		<div v-else class="flex flex-col gap-1">
			<div
				v-for="instance in filteredInstances"
				:key="instance.id"
				class="flex items-center justify-between gap-2 px-6 py-1.5 hover:bg-surface-3"
			>
				<div class="flex min-w-0 items-center gap-2.5 overflow-hidden">
					<Avatar :src="instance.iconUrl" size="2rem" rounded="md" pad-transparent-corners />
					<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
				</div>
				<Button :disabled="ctx.finishDisabled.value" @click="select(instance)">
					{{ formatMessage(messages.selectButton) }}
				</Button>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { computed, onMounted, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectInstanceServerSource, type ServerInstanceOption } from '../../../../providers'
import Avatar from '../../../base/Avatar.vue'
import { Button } from '../../../base/buttons'
import Input from '../../../base/inputs/Input.vue'
import LoadingIndicator from '../../../base/LoadingIndicator.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

/** Server setup: pick the instance the server is created from. Same list as adding a mod to an instance. */
const ctx = injectCreationFlowContext()
const instanceSource = injectInstanceServerSource(null)
const { formatMessage } = useVIntl()

const instances = ref<ServerInstanceOption[]>([])
const loading = ref(true)
const searchFilter = ref('')

const filteredInstances = computed(() => {
	const query = searchFilter.value.trim().toLowerCase()
	return instances.value.filter((instance) => instance.name.toLowerCase().includes(query))
})

onMounted(async () => {
	try {
		instances.value = (await instanceSource?.listInstances()) ?? []
	} finally {
		loading.value = false
	}
})

function select(instance: ServerInstanceOption) {
	if (ctx.finishDisabled.value) return
	ctx.instanceSelection.value = instance
	ctx.modal.value?.setStage('final-config')
}

const messages = defineMessages({
	searchPlaceholder: {
		id: 'creation-flow.modal.select-instance.search-placeholder',
		defaultMessage: 'Search instance',
	},
	noInstances: {
		id: 'creation-flow.modal.select-instance.no-instances',
		defaultMessage: 'No instances found',
	},
	selectButton: {
		id: 'creation-flow.modal.select-instance.select-button',
		defaultMessage: 'Select',
	},
})
</script>
