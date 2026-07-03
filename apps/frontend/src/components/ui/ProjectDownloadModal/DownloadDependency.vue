<template>
	<div class="flex min-w-0 flex-col gap-2">
		<div
			class="grid min-h-10 grid-cols-[minmax(0,1fr)_min-content] items-center gap-3 rounded-xl bg-button-bg py-0 pl-3.5 pr-2 text-primary"
		>
			<span class="flex min-w-0 items-center gap-3">
				<Avatar
					v-if="dependency.icon"
					:src="dependency.icon"
					:alt="dependency.name"
					size="24px"
					class="!rounded-lg !shadow-none"
				/>
				<span
					v-else
					class="flex size-4 flex-shrink-0 items-center justify-center rounded-lg border border-solid border-surface-5 text-secondary"
				>
					<PackageIcon aria-hidden="true" class="size-5" />
				</span>
				<a
					v-if="dependency.projectHref"
					ref="dependencyNameRef"
					v-tooltip="truncatedTooltip(dependencyNameRef, dependency.name)"
					:href="dependency.projectHref"
					target="_blank"
					rel="noopener noreferrer"
					class="min-w-0 truncate text-base font-semibold text-contrast no-underline hover:underline"
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
				<TagItem class="shrink-0 border !border-solid border-surface-5 !px-3 !py-1 text-base">
					{{ dependency.typeLabel }}
				</TagItem>
			</span>
			<ButtonStyled v-if="dependency.downloadHref" circular type="transparent">
				<a
					v-tooltip="'Download'"
					:href="dependency.downloadHref"
					:download="dependency.filename"
					:aria-label="`Download ${dependency.name}`"
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
			class="group/dependency relative pl-10"
		>
			<div
				aria-hidden="true"
				class="absolute -top-2 left-6 h-16 w-0.5 bg-surface-5 group-last/dependency:h-7"
			/>
			<div aria-hidden="true" class="absolute left-6 top-5 h-0.5 w-4 bg-surface-5" />
			<DownloadDependency :dependency="childDependency" @download="emit('download')" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon, PackageIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, TagItem, truncatedTooltip } from '@modrinth/ui'
import { ref } from 'vue'

defineOptions({
	name: 'DownloadDependency',
})

interface DownloadDependencyRow {
	key: string
	name: string
	icon?: string
	projectHref?: string
	downloadHref?: string
	filename?: string
	typeLabel: string
	unavailableTooltip: string
	dependencies: DownloadDependencyRow[]
}

defineProps<{
	dependency: DownloadDependencyRow
}>()

const emit = defineEmits<{
	download: []
}>()

const dependencyNameRef = ref<HTMLElement | null>(null)
</script>
