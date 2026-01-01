<template>
	<div
		class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-2 text-button-text"
	>
		<div class="flex items-center gap-2 overflow-hidden">
			<FileIcon v-if="isPrimary" class="text-lg" />
			<FilePlusIcon v-else class="text-lg" />

			<span v-tooltip="name" class="overflow-hidden text-ellipsis whitespace-nowrap font-medium">
				{{ name }}
			</span>

			<TagItem class="shrink-0 border !border-solid border-surface-5">
				{{ isPrimary ? 'Primary' : fileTypeLabels[fileType ?? 'unknown'] }}
			</TagItem>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { FileIcon, FilePlusIcon } from '@modrinth/assets'
import { TagItem } from '@modrinth/ui'

const { name, isPrimary, fileType } = defineProps<{
	name: string
	isPrimary?: boolean
	fileType?: Labrinth.Versions.v3.FileType | 'primary'
}>()

const fileTypeLabels: Record<Labrinth.Versions.v3.FileType | 'primary', string> = {
	primary: 'Primary',
	unknown: 'Other',
	'required-resource-pack': 'Required RP',
	'optional-resource-pack': 'Optional RP',
	'sources-jar': 'Sources JAR',
	'dev-jar': 'Dev JAR',
	'javadoc-jar': 'Javadoc JAR',
	signature: 'Signature',
}
</script>
