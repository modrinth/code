<template>
	<Accordion
		overflow-visible
		button-class="w-full bg-transparent m-0 p-0 border-none"
		@on-open="forceCheck"
	>
		<template #title>
			<FileCogIcon class="size-4 shrink-0" />
			<span class="text-lg font-semibold text-contrast">{{ formatMessage(messages.title) }}</span>
		</template>
		<div class="flex min-w-0 flex-col gap-3 pt-4">
			<Admonition
				v-if="directory.isError.value"
				type="critical"
				:header="formatMessage(messages.error)"
			>
				<Button @click="directory.refetch()">{{ formatMessage(messages.retry) }}</Button>
			</Admonition>
			<div
				v-show="directory.data.value"
				class="relative overflow-hidden rounded-[20px]"
				:inert="disabled"
			>
				<div ref="container" class="max-h-[292px] overflow-y-auto">
					<FileTreeSelect
						v-model="includedPaths"
						v-model:excluded-paths="excludedPaths"
						:items="directory.data.value ?? []"
						lazy
						:show-size="false"
						:show-modified="false"
						@navigate="navigate"
					/>
				</div>
				<div
					v-if="showTopFade"
					aria-hidden="true"
					class="pointer-events-none absolute inset-x-0 top-0 z-10 h-6 bg-gradient-to-b from-surface-3 to-transparent"
				/>
				<div
					v-if="showBottomFade"
					aria-hidden="true"
					class="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-6 bg-gradient-to-t from-surface-3 to-transparent"
				/>
			</div>
		</div>
	</Accordion>
</template>

<script setup lang="ts">
import { FileCogIcon } from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import Accordion from '#ui/components/base/Accordion.vue'
import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import FileTreeSelect, { type FileTreeSelectItem } from '#ui/components/base/FileTreeSelect.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useScrollIndicator } from '#ui/composables/scroll-indicator'
import { injectAuth, injectModrinthClient } from '#ui/providers'

const props = defineProps<{ serverId: string; worldId: string; disabled?: boolean }>()
const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const auth = injectAuth()
const queryClient = useQueryClient()
const currentPath = ref('')
const includedPaths = ref<string[]>([])
const excludedPaths = ref<string[]>([])
const container = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, forceCheck } = useScrollIndicator(container)
const extensions = new Set([
	'json',
	'json5',
	'jsonc',
	'yml',
	'yaml',
	'css',
	'toml',
	'txt',
	'ini',
	'cfg',
	'conf',
	'properties',
	'xml',
	'nbt',
])

function directoryOptions(path: string) {
	const worldId = props.worldId
	const directoryPath = path ? `/config/${path}` : '/config'
	return {
		queryKey: [
			'servers',
			'share-config-files',
			props.serverId,
			worldId,
			auth.user.value?.id,
			directoryPath,
		],
		queryFn: async () => {
			const items: FileTreeSelectItem[] = []
			let pages = 1
			for (let page = 1; page <= pages; page++) {
				const result = await client.kyros.files_v0.listDirectory(directoryPath, page, 2000)
				pages = result.total
				for (const item of result.items) {
					if (item.name.startsWith('.') || /[/\\:]/.test(item.name) || /[. ]$/.test(item.name))
						continue
					if (item.type !== 'directory' && item.type !== 'file') continue
					if (
						item.type === 'file' &&
						!extensions.has(item.name.split('.').pop()?.toLowerCase() ?? '')
					)
						continue
					items.push({
						path: path ? `${path}/${item.name}` : item.name,
						type: item.type,
						size: item.size,
					})
				}
			}
			return items
		},
		staleTime: 0,
		retry: false,
	}
}

const directory = useQuery(computed(() => directoryOptions(currentPath.value)))

function navigate(path: string) {
	currentPath.value = path
	if (container.value) container.value.scrollTop = 0
}

async function resolvePaths() {
	const included = new Set(includedPaths.value)
	const excluded = new Set(excludedPaths.value)
	if (!included.size) return []
	const selected: string[] = []
	let totalSize = 0
	const worldId = props.worldId
	const userId = auth.user.value?.id
	function isSelected(path: string) {
		let selected = false
		let prefix = ''
		for (const segment of path.split('/')) {
			prefix = prefix ? `${prefix}/${segment}` : segment
			if (included.has(prefix)) selected = true
			if (excluded.has(prefix)) selected = false
		}
		return selected
	}
	async function visit(path: string) {
		if (props.worldId !== worldId || auth.user.value?.id !== userId)
			throw new Error(formatMessage(messages.contextChanged))
		const items = await queryClient.fetchQuery(directoryOptions(path))
		for (const item of items) {
			if (item.type === 'directory') {
				if (
					isSelected(item.path) ||
					[...included].some((included) => included.startsWith(`${item.path}/`))
				)
					await visit(item.path)
			} else if (isSelected(item.path)) {
				const size = item.size ?? 0
				totalSize += size
				if (size > 16 * 1024 * 1024 || totalSize > 128 * 1024 * 1024 || selected.length >= 4096) {
					throw new Error(formatMessage(messages.selectionTooLarge))
				}
				selected.push(`config/${item.path}`)
			}
		}
	}
	await visit('')
	return selected
}

const messages = defineMessages({
	title: {
		id: 'instance.shared-instance.publish-review.config-title-v2',
		defaultMessage: 'Select config files',
	},
	error: { id: 'servers.play.config-files.error', defaultMessage: 'Could not load config files' },
	retry: { id: 'servers.play.retry', defaultMessage: 'Retry' },
	selectionTooLarge: {
		id: 'servers.play.config-files.selection-too-large',
		defaultMessage:
			'Select up to 4,096 config files, with a maximum of 16 MiB per file and 128 MiB in total.',
	},
	contextChanged: {
		id: 'servers.play.config-files.context-changed',
		defaultMessage: 'The world or account changed. Open Push update again.',
	},
})

defineExpose({ resolvePaths })
</script>
