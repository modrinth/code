<script setup lang="ts">
import { computed } from 'vue'

import Combobox from '#ui/components/base/Combobox.vue'
import VersionChannelIndicator from '#ui/components/version/VersionChannelIndicator.vue'

import type { UpdateAllVersion } from './update-all-modal-types'

const props = defineProps<{
	versions: UpdateAllVersion[]
	version: UpdateAllVersion
	label: string
	disabled?: boolean
}>()

const emit = defineEmits<{
	select: [versionId: string]
}>()

const options = computed(() =>
	props.versions.map((version) => ({ value: version.id, label: version.version_number })),
)

function channelClasses(channel: UpdateAllVersion['version_type']) {
	return {
		'!bg-highlight-green !text-green': channel === 'release',
		'!bg-highlight-orange !text-orange': channel === 'beta',
		'!bg-highlight-purple !text-purple': channel === 'alpha',
	}
}

function channelFor(versionId: string) {
	return props.versions.find((candidate) => candidate.id === versionId)?.version_type ?? props.version.version_type
}
</script>

<template>
	<Combobox
		v-if="versions.length > 1"
		class="!w-auto min-w-0 max-w-full"
		:model-value="version.id"
		:options="options"
		:disabled="disabled"
		trigger-size="sm"
		trigger-class="!gap-1.5 !rounded-xl !px-3 !font-medium"
		:dropdown-min-width="240"
		@update:model-value="emit('select', $event)"
	>
		<template #prefix>
			<VersionChannelIndicator
				:channel="version.version_type"
				class="!size-6 shrink-0 !font-medium"
				:class="channelClasses(version.version_type)"
			/>
		</template>
		<template #selected>
			<span class="sr-only">{{ label }}: </span>
			<span class="font-medium">{{ version.version_number }}</span>
		</template>
		<template #option="{ item }">
			<VersionChannelIndicator
				:channel="channelFor(item.value)"
				class="!size-6 shrink-0 !font-medium"
				:class="channelClasses(channelFor(item.value))"
			/>
			<span>{{ item.label }}</span>
		</template>
	</Combobox>
	<div v-else class="flex h-8 min-w-0 items-center gap-1.5 text-sm font-medium text-contrast">
		<VersionChannelIndicator
			:channel="version.version_type"
			class="!size-6 shrink-0 !font-medium"
			:class="channelClasses(version.version_type)"
		/>
		<span class="truncate" :title="version.version_number">{{ version.version_number }}</span>
	</div>
</template>
