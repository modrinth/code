<template>
	<div
		class="flex max-h-[272px] flex-col overflow-y-auto border-0 border-y border-solid border-surface-5 bg-surface-2 px-3 py-2"
	>
		<div v-if="!diffs.length" class="flex h-10 min-h-10 items-center gap-2 px-2">
			<div class="flex w-4 shrink-0 items-center justify-center">
				<MinusIcon class="size-4" />
			</div>
			<span class="text-sm font-medium text-contrast">
				{{ formatMessage(messages.noContentChanges) }}
			</span>
		</div>
		<template
			v-for="(diff, index) in sortedDiffs"
			:key="diff.projectName || diff.fileName || (isConfigurationDiff(diff) ? diff.type : index)"
		>
			<component
				:is="hasConfigFolders(diff) ? 'button' : 'div'"
				:type="hasConfigFolders(diff) ? 'button' : undefined"
				:aria-expanded="hasConfigFolders(diff) ? configFoldersExpanded : undefined"
				class="flex h-10 min-h-10 shrink-0 items-center gap-2 border-none bg-transparent p-0 text-left text-[inherit]"
				:class="[
					showExternalWarning(diff) ? '-mx-3 px-5' : 'px-2',
					{ 'cursor-pointer rounded-lg hover:bg-surface-3': hasConfigFolders(diff) },
				]"
				:style="
					showExternalWarning(diff)
						? {
								backgroundColor: 'color-mix(in srgb, var(--color-orange) 10%, transparent)',
							}
						: undefined
				"
				@click="toggleConfigFolders(diff)"
			>
				<div class="relative flex w-4 shrink-0 self-stretch items-center justify-center">
					<div
						v-if="index > 0"
						class="absolute left-1/2 top-0 h-3 w-px -translate-x-1/2 bg-surface-5"
					/>
					<PlusIcon
						v-if="diff.type === 'added' || diff.type === 'modpack_linked'"
						class="relative z-[1] size-4"
					/>
					<MinusIcon
						v-else-if="diff.type === 'removed' || diff.type === 'modpack_unlinked'"
						class="relative z-[1] size-4 text-red"
					/>
					<FileCogIcon
						v-else-if="diff.type === 'config_files_updated'"
						class="relative z-[1] size-4"
					/>
					<RefreshCwIcon v-else class="relative z-[1] size-4" />
					<div
						v-if="index < sortedDiffs.length - 1 || isConfigFoldersOpen(diff)"
						class="absolute bottom-0 left-1/2 top-7 w-px -translate-x-1/2 bg-surface-5"
					/>
				</div>

				<div class="flex min-w-0 flex-1 items-center gap-1 text-sm">
					<span class="shrink-0 whitespace-nowrap text-primary">{{ getDiffTypeLabel(diff) }}</span>
					<template v-if="showExternalWarning(diff)">
						<CircleAlertIcon class="size-4 shrink-0 text-orange" />
						<span class="truncate font-medium text-orange">
							{{ formatMessage(messages.unknownProject) }}
						</span>
					</template>
					<span
						v-else-if="!isConfigurationDiff(diff) || diff.type === 'modpack_updated'"
						class="truncate font-medium text-contrast"
					>
						{{ diff.projectName || (diff.fileName ? decodeURIComponent(diff.fileName) : '') }}
					</span>
				</div>
				<span
					v-if="getVersionLabel(diff)"
					class="ml-2 max-w-[60%] min-w-0 shrink truncate text-right text-xs"
					:class="showExternalWarning(diff) ? 'text-orange' : 'text-primary'"
					:title="getVersionLabel(diff)"
				>
					{{ getVersionLabel(diff) }}
				</span>
				<ChevronDownIcon
					v-if="hasConfigFolders(diff)"
					class="size-4 shrink-0 text-secondary transition-transform"
					:class="{ 'rotate-180': configFoldersExpanded }"
				/>
			</component>

			<template v-if="isConfigFoldersOpen(diff)">
				<div
					v-for="(folder, folderIndex) in configFolders"
					:key="folder.path"
					class="flex h-8 min-h-8 shrink-0 items-center gap-2 px-2"
				>
					<div class="relative flex w-4 shrink-0 self-stretch justify-center">
						<div
							class="absolute left-1/2 top-0 w-px -translate-x-1/2 bg-surface-5"
							:class="
								folderIndex < configFolders.length - 1 || index < sortedDiffs.length - 1
									? 'bottom-0'
									: 'h-1/2'
							"
						/>
					</div>
					<FolderIcon class="size-4 shrink-0 text-secondary" />
					<span class="min-w-0 flex-1 truncate text-sm text-primary" :title="folder.path">
						{{ folder.path || formatMessage(messages.instanceFolder) }}
					</span>
					<span class="shrink-0 text-xs text-secondary tabular-nums">
						{{ formatMessage(messages.fileCount, { count: folder.count }) }}
					</span>
				</div>
			</template>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronDownIcon,
	CircleAlertIcon,
	FileCogIcon,
	FolderIcon,
	MinusIcon,
	PlusIcon,
	RefreshCwIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ContentDiffItem } from '../types'
import { isConfigurationDiff, isDependencyDiff, showsExternalWarning } from '../utils'

const props = defineProps<{
	diffs: ContentDiffItem[]
	addedLabel?: string
	removedLabel?: string
	showExternalWarnings?: boolean
}>()

const { formatMessage } = useVIntl()

function showExternalWarning(diff: ContentDiffItem) {
	return showsExternalWarning(diff, props.showExternalWarnings)
}

const sortedDiffs = computed(() =>
	[...props.diffs].sort((a, b) => {
		const aExternal = showExternalWarning(a)
		const bExternal = showExternalWarning(b)
		if (aExternal !== bExternal) return aExternal ? -1 : 1

		const typeOrder: Record<ContentDiffItem['type'], number> = {
			modpack_linked: 0,
			modpack_updated: 0,
			modpack_unlinked: 0,
			game_version_updated: 1,
			loader_updated: 2,
			added: 3,
			updated: 4,
			removed: 5,
			config_files_updated: 6,
		}
		return typeOrder[a.type] - typeOrder[b.type]
	}),
)

const CONFIG_FOLDER_DEPTH = 2

const configFoldersExpanded = ref(false)

const configFolders = computed(() => {
	const counts = new Map<string, number>()
	for (const fileName of props.diffs.find(isConfigFilesDiff)?.fileNames ?? []) {
		const folder = fileName.split('/').slice(0, -1).slice(0, CONFIG_FOLDER_DEPTH).join('/')
		counts.set(folder, (counts.get(folder) ?? 0) + 1)
	}
	return [...counts]
		.map(([path, count]) => ({ path, count }))
		.sort((a, b) => b.count - a.count || a.path.localeCompare(b.path))
})

function isConfigFilesDiff(diff: ContentDiffItem) {
	return diff.type === 'config_files_updated'
}

function hasConfigFolders(diff: ContentDiffItem) {
	return isConfigFilesDiff(diff) && configFolders.value.length > 0
}

function isConfigFoldersOpen(diff: ContentDiffItem) {
	return hasConfigFolders(diff) && configFoldersExpanded.value
}

function toggleConfigFolders(diff: ContentDiffItem) {
	if (hasConfigFolders(diff)) configFoldersExpanded.value = !configFoldersExpanded.value
}

function getDiffTypeLabel(diff: ContentDiffItem) {
	if (showExternalWarning(diff) && isDependencyDiff(diff)) {
		return formatMessage(externalDiffTypeMessages[diff.type])
	}
	if (diff.type === 'modpack_updated') return formatMessage(diffTypeMessages.updated)
	if (isConfigurationDiff(diff)) return formatMessage(configurationDiffMessages[diff.type])
	if (diff.type === 'removed' && diff.disabled) {
		return formatMessage(diffTypeMessages.removedDisabled)
	}
	if (diff.type === 'added' && props.addedLabel) return props.addedLabel
	if (diff.type === 'removed' && props.removedLabel) return props.removedLabel
	return formatMessage(diffTypeMessages[diff.type])
}

function getVersionLabel(diff: ContentDiffItem) {
	if (showExternalWarning(diff) && diff.fileName) return decodeURIComponent(diff.fileName)
	if (diff.type === 'config_files_updated' && diff.fileCount !== undefined) {
		return formatMessage(messages.fileCount, { count: diff.fileCount })
	}
	if (diff.type === 'modpack_updated') return diff.newVersionName
	if (isConfigurationDiff(diff)) {
		if (diff.currentVersionName && diff.newVersionName) {
			return `${diff.currentVersionName} → ${diff.newVersionName}`
		}
		return diff.newVersionName ?? diff.currentVersionName
	}
	return diff.type === 'removed' ? diff.currentVersionName : diff.newVersionName
}

const messages = defineMessages({
	noContentChanges: {
		id: 'content.diff-modal.no-content-changes',
		defaultMessage: 'No content changes',
	},
	fileCount: {
		id: 'content.diff-modal.file-count',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	unknownProject: {
		id: 'content.diff-modal.unknown-project',
		defaultMessage: 'Unknown',
	},
	instanceFolder: {
		id: 'content.diff-modal.instance-folder',
		defaultMessage: 'Instance folder',
	},
})

const configurationDiffMessages = defineMessages({
	modpack_linked: {
		id: 'content.diff-modal.modpack-linked',
		defaultMessage: 'Linked modpack',
	},
	modpack_updated: {
		id: 'content.diff-modal.modpack-updated',
		defaultMessage: 'Updated modpack',
	},
	modpack_unlinked: {
		id: 'content.diff-modal.modpack-unlinked',
		defaultMessage: 'Unlinked modpack',
	},
	game_version_updated: {
		id: 'content.diff-modal.game-version-updated',
		defaultMessage: 'Game version',
	},
	loader_updated: {
		id: 'content.diff-modal.loader-updated',
		defaultMessage: 'Loader',
	},
	config_files_updated: {
		id: 'content.diff-modal.config-files-updated',
		defaultMessage: 'Config files',
	},
})

const diffTypeMessages = defineMessages({
	added: {
		id: 'content.diff-modal.diff-type.added',
		defaultMessage: 'Added (dependency)',
	},
	removed: {
		id: 'content.diff-modal.diff-type.removed',
		defaultMessage: 'Disabled',
	},
	removedDisabled: {
		id: 'content.diff-modal.diff-type.removed-disabled',
		defaultMessage: 'Removed (disabled)',
	},
	updated: {
		id: 'content.diff-modal.diff-type.updated',
		defaultMessage: 'Updated',
	},
})

const externalDiffTypeMessages = defineMessages({
	added: {
		id: 'content.diff-modal.external-diff-type.added',
		defaultMessage: 'Added',
	},
	removed: {
		id: 'content.diff-modal.external-diff-type.removed',
		defaultMessage: 'Removed',
	},
	updated: {
		id: 'content.diff-modal.external-diff-type.updated',
		defaultMessage: 'Updated',
	},
})
</script>
