<script setup lang="ts">
import { computed } from 'vue'

import Combobox from '#ui/components/base/Combobox.vue'
import VersionChannelIndicator from '#ui/components/version/VersionChannelIndicator.vue'

import type { UpdateAllVersion } from './update-all-modal-types'
import { useUpdateAllTruncatedTooltip } from './use-update-all-truncated-tooltip'

const props = defineProps<{
	versions: UpdateAllVersion[]
	version: UpdateAllVersion
	label: string
	disabled?: boolean
	wrap?: boolean
}>()

const emit = defineEmits<{
	select: [versionId: string]
}>()

const options = computed(() =>
	props.versions.map((version) => ({ value: version.id, label: version.version_number })),
)
const { element: versionLabel, tooltip } = useUpdateAllTruncatedTooltip(
	() => props.version.version_number,
	() => !props.wrap || props.versions.length > 1,
)

function channelClasses(channel: UpdateAllVersion['version_type']) {
	return {
		'!bg-highlight-green !text-green': channel === 'release',
		'!bg-highlight-orange !text-orange': channel === 'beta',
		'!bg-highlight-purple !text-purple': channel === 'alpha',
	}
}

function channelFor(versionId: string) {
	return (
		props.versions.find((candidate) => candidate.id === versionId)?.version_type ??
		props.version.version_type
	)
}
</script>

<template>
	<div v-if="versions.length > 1" v-tooltip="tooltip" class="min-w-0 max-w-full">
		<Combobox
			:model-value="version.id"
			:options="options"
			:disabled="disabled"
			:animate-dropdown="false"
			trigger-size="sm"
			trigger-class="!gap-1.5 !rounded-xl !px-3 !font-medium"
			:dropdown-min-width="240"
			@update:model-value="emit('select', $event)"
		>
			<template #prefix>
				<VersionChannelIndicator
					v-if="version.version_type"
					:channel="version.version_type ?? 'release'"
					class="!size-6 shrink-0 !font-medium"
					:class="channelClasses(version.version_type)"
				/>
			</template>
			<template #selected>
				<span class="sr-only">{{ label }}: </span>
				<span
					ref="versionLabel"
					class="inline-block max-w-full truncate align-middle font-medium"
					>{{ version.version_number }}</span
				>
			</template>
			<template #option="{ item }">
				<VersionChannelIndicator
					v-if="channelFor(item.value)"
					:channel="channelFor(item.value) ?? 'release'"
					class="!size-6 shrink-0 !font-medium"
					:class="channelClasses(channelFor(item.value))"
				/>
				<span class="min-w-0 break-all">{{ item.label }}</span>
			</template>
		</Combobox>
	</div>
	<div
		v-else
		v-tooltip="tooltip"
		class="flex min-h-8 min-w-0 items-center gap-1.5 pl-3 text-sm font-medium text-contrast"
	>
		<VersionChannelIndicator
			v-if="version.version_type"
			:channel="version.version_type ?? 'release'"
			class="!size-6 shrink-0 !font-medium"
			:class="channelClasses(version.version_type)"
		/>
		<span ref="versionLabel" class="min-w-0" :class="wrap ? 'break-all' : 'block truncate'">
			{{ version.version_number }}
		</span>
	</div>
</template>
