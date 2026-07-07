<template>
	<div class="flex min-w-0 flex-col">
		<div
			class="z-10 grid h-11 grid-cols-[minmax(0,1fr)_min-content] items-center gap-3 text-primary"
		>
			<span class="flex min-w-0 items-center gap-2">
				<Avatar
					v-if="dependency.icon"
					:src="dependency.icon"
					:alt="dependency.name"
					size="24px"
					class="!rounded-lg !shadow-none"
				/>
				<span
					v-else
					class="flex size-6 flex-shrink-0 items-center justify-center rounded-lg border border-solid border-surface-5 text-secondary"
				>
					<component
						:is="dependency.fallbackIcon ?? PackageIcon"
						aria-hidden="true"
						class="size-4"
					/>
				</span>
				<a
					v-if="dependency.projectHref"
					ref="dependencyNameRef"
					v-tooltip="truncatedTooltip(dependencyNameRef, dependency.name)"
					:href="dependency.projectHref"
					target="_blank"
					rel="noopener noreferrer"
					class="min-w-0 truncate text-base font-semibold text-contrast hover:underline"
				>
					{{ dependency.name }}
				</a>
				<span
					v-else
					ref="dependencyNameRef"
					v-tooltip="truncatedTooltip(dependencyNameRef, dependency.name)"
					class="min-w-0 truncate text-base font-semibold text-contrast"
				>
					{{ dependency.name }}
				</span>
				<span
					v-tooltip="metadataTooltip"
					class="min-w-0 max-w-[50%] truncate text-sm text-secondary"
				>
					{{ metadataLabel }}
				</span>
			</span>
			<ButtonStyled v-if="dependency.downloadHref" circular type="transparent">
				<a
					v-tooltip="downloadTooltip"
					:href="dependency.downloadHref"
					:download="dependency.filename"
					:aria-label="downloadTooltip"
					@click="emit('download')"
				>
					<DownloadIcon aria-hidden="true" class="size-6 text-secondary" />
				</a>
			</ButtonStyled>
			<ButtonStyled v-else circular type="transparent">
				<button
					v-tooltip="dependency.unavailableTooltip"
					disabled
					:aria-label="dependency.unavailableTooltip"
				>
					<DownloadIcon aria-hidden="true" class="size-6 text-secondary" />
				</button>
			</ButtonStyled>
		</div>
		<div
			v-for="childDependency in dependency.dependencies"
			:key="childDependency.key"
			class="group/dependency relative pl-8"
		>
			<DownloadDependency
				:dependency="childDependency"
				class="relative z-10"
				@download="emit('download')"
			/>
			<div
				aria-hidden="true"
				class="absolute -top-2.5 left-3 z-0 h-full w-0.5 bg-surface-5 group-last/dependency:h-8"
			/>
			<div aria-hidden="true" class="absolute left-3 top-[21px] z-0 h-0.5 w-7 bg-surface-5" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon, PackageIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	defineMessages,
	truncatedTooltip,
	useFormatBytes,
	useVIntl,
} from '@modrinth/ui'
import { type Component, computed, ref } from 'vue'

defineOptions({
	name: 'DownloadDependency',
})

interface DownloadDependencyRow {
	key: string
	name: string
	icon?: string
	fallbackIcon?: Component
	projectHref?: string
	downloadHref?: string
	filename?: string
	fileSize?: number
	metadataLabel?: string
	typeLabel: string
	unavailableTooltip: string
	dependencies: DownloadDependencyRow[]
}

const props = defineProps<{
	dependency: DownloadDependencyRow
}>()

const emit = defineEmits<{
	download: []
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const dependencyNameRef = ref<HTMLElement | null>(null)

const metadataLabel = computed(() => props.dependency.metadataLabel ?? props.dependency.typeLabel)
const metadataTooltip = computed(() =>
	metadataLabel.value === props.dependency.typeLabel ? null : metadataLabel.value,
)

const downloadTooltip = computed(() => {
	const filename = props.dependency.filename || props.dependency.name

	if (typeof props.dependency.fileSize === 'number') {
		return formatMessage(messages.downloadFileWithSize, {
			filename,
			size: formatBytes(props.dependency.fileSize, 1),
		})
	}

	return formatMessage(messages.downloadFile, { filename })
})

const messages = defineMessages({
	downloadFile: {
		id: 'project.download.dependency-download-file',
		defaultMessage: 'Download {filename}',
	},
	downloadFileWithSize: {
		id: 'project.download.dependency-download-file-with-size',
		defaultMessage: 'Download {filename} ({size})',
	},
})
</script>
