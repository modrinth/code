<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ClipboardCopyIcon, DownloadIcon, LoaderCircleIcon, XIcon } from '@modrinth/assets'
import { Button, ButtonLink, CopyCode, injectModrinthClient, NewModal } from '@modrinth/ui'
import { useQueries } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef } from 'vue'

import { versionQueryOptions } from '~/composables/queries/version'

export type UnsafeFile = {
	file: Labrinth.TechReview.Internal.FileReport & { version_id: string }
	projectName: string
	projectId: string
	userId: string
	username: string
}

const props = defineProps<{
	unsafeFiles: UnsafeFile[]
}>()

const client = injectModrinthClient()
const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const opened = ref(false)
const versionIds = computed(() => [
	...new Set(props.unsafeFiles.map(({ file }) => file.version_id)),
])
const versions = useQueries({
	queries: computed(() =>
		versionIds.value.map((id) => ({
			...versionQueryOptions.v3(id, client),
			enabled: opened.value,
		})),
	),
})

function getFileHash(versionId: string, fileId: string): string | undefined {
	return versions.value
		.find(({ data }) => data?.id === versionId)
		?.data?.files.find(({ id }) => id === fileId)?.hashes.sha512
}

function isHashLoading(versionId: string): boolean {
	return versions.value[versionIds.value.indexOf(versionId)]?.isPending ?? false
}

let resolveClosed: (() => void) | undefined

function show(): Promise<void> {
	opened.value = true
	modalRef.value?.show()
	return new Promise((resolve) => {
		resolveClosed = resolve
	})
}

function onHide() {
	opened.value = false
	resolveClosed?.()
	resolveClosed = undefined
}

function hide() {
	modalRef.value?.hide()
}

async function copy(text: string) {
	await navigator.clipboard.writeText(text)
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modalRef"
		header="Malicious file(s) summary"
		:on-hide="onHide"
		:close-on-click-outside="false"
		:close-on-esc="false"
		:closable="false"
	>
		<div class="markdown-body inset-0">
			<div v-if="unsafeFiles.length > 0" class="mb-4 flex flex-col gap-2">
				<div class="flex items-center gap-2">
					<span class="text-tertiary text-sm font-medium">Project:</span>
					<CopyCode :text="unsafeFiles[0].projectName" />
					<CopyCode :text="unsafeFiles[0].projectId" />
				</div>
				<div class="flex items-center gap-2">
					<span class="text-tertiary text-sm font-medium">User:</span>
					<CopyCode :text="unsafeFiles[0].username" />
					<CopyCode :text="unsafeFiles[0].userId" />
				</div>
			</div>

			<table v-if="unsafeFiles.length > 0" class="w-full text-sm">
				<thead>
					<tr class="text-tertiary text-left text-xs font-medium">
						<th class="pb-2">Hash</th>
						<th class="pb-2">Version ID</th>
						<th class="pb-2">File Name</th>
						<th class="pb-2">CDN Link</th>
						<th class="pb-2">Download</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="item in unsafeFiles" :key="item.file.file_id">
						<td class="py-1 pr-2">
							<LoaderCircleIcon
								v-if="isHashLoading(item.file.version_id)"
								class="size-4 animate-spin text-secondary"
							/>
							<Button
								v-else-if="getFileHash(item.file.version_id, item.file.file_id)"
								class="!h-6"
								size="xs"
								@click="copy(getFileHash(item.file.version_id, item.file.file_id)!)"
							>
								<ClipboardCopyIcon class="size-4" />
								Copy
							</Button>
							<span v-else class="text-tertiary italic">N/A</span>
						</td>
						<td class="py-1 pr-2">
							<CopyCode :text="item.file.version_id" />
						</td>
						<td class="py-1 pr-2">
							<CopyCode :text="item.file.file_name" />
						</td>
						<td class="py-1 pr-2">
							<Button class="!h-6" size="xs" @click="copy(item.file.download_url)">
								<ClipboardCopyIcon class="size-4" />
								Copy
							</Button>
						</td>
						<td class="py-1">
							<ButtonLink
								size="xs"
								:href="item.file.download_url"
								:download="item.file.file_name"
								target="_blank"
								class="!size-6 !h-6 !rounded-full !px-0"
							>
								<DownloadIcon />
							</ButtonLink>
						</td>
					</tr>
				</tbody>
			</table>

			<p v-else class="text-sm italic text-secondary">No files currently marked as malicious.</p>

			<div class="flex justify-end">
				<Button @click="hide">
					<XIcon class="size-4" />
					Close
				</Button>
			</div>
		</div>
	</NewModal>
</template>
