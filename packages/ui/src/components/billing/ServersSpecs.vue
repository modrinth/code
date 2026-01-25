<script setup lang="ts">
import { CpuIcon, DatabaseIcon, MemoryStickIcon, SparklesIcon, UnknownIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import AutoLink from '../base/AutoLink.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	gbRam: {
		id: 'hosting.specs.gb-ram',
		defaultMessage: '{ram} GB RAM',
	},
	gbStorage: {
		id: 'hosting.specs.gb-storage',
		defaultMessage: '{storage} GB Storage',
	},
	sharedCpus: {
		id: 'hosting.specs.shared-cpus',
		defaultMessage: '{cpus} Shared CPUs',
	},
	burst: {
		id: 'hosting.specs.burst',
		defaultMessage: 'Bursts up to {cpus} CPUs',
	},
	burstTooltip: {
		id: 'hosting.specs.burst.tooltip',
		defaultMessage:
			'CPU bursting allows your server to temporarily use additional threads to help mitigate TPS spikes. Click for more info.',
	},
})

const emit = defineEmits<{
	(e: 'click-bursting-link'): void
}>()

const props = defineProps<{
	ram: number
	storage: number
	cpus: number
}>()

const formattedRam = computed(() => {
	return props.ram / 1024
})

const formattedStorage = computed(() => {
	return props.storage / 1024
})

const sharedCpus = computed(() => {
	return props.cpus / 2
})
</script>
<template>
	<ul class="m-0 flex list-none flex-col gap-2 px-0 text-sm leading-normal text-secondary">
		<li class="flex items-center gap-2">
			<MemoryStickIcon class="h-5 w-5 shrink-0" />
			{{ formatMessage(messages.gbRam, { ram: formattedRam }) }}
		</li>
		<li class="flex items-center gap-2">
			<DatabaseIcon class="h-5 w-5 shrink-0" />
			{{ formatMessage(messages.gbStorage, { storage: formattedStorage }) }}
		</li>
		<li class="flex items-center gap-2">
			<CpuIcon class="h-5 w-5 shrink-0" />
			{{ formatMessage(messages.sharedCpus, { cpus: sharedCpus }) }}
		</li>
		<li class="flex items-center gap-2">
			<SparklesIcon class="h-5 w-5 shrink-0" />
			{{ formatMessage(messages.burst, { cpus: props.cpus }) }}
			<AutoLink
				v-tooltip="formatMessage(messages.burstTooltip)"
				class="flex"
				to="https://modrinth.com/hosting#cpu-burst"
				target="_blank"
				@click="() => emit('click-bursting-link')"
			>
				<UnknownIcon class="h-4 w-4 text-secondary opacity-80" />
			</AutoLink>
		</li>
	</ul>
</template>
