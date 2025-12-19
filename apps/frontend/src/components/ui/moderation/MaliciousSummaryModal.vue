<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ClipboardCopyIcon, LoaderCircleIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, CopyCode, NewModal } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

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

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')

const versionDataCache = ref<
	Map<
		string,
		{
			files: Map<string, string>
			loading: boolean
			error?: string
		}
	>
>(new Map())

async function fetchVersionHashes(versionIds: string[]) {
	const uniqueIds = [...new Set(versionIds)]
	for (const versionId of uniqueIds) {
		if (versionDataCache.value.has(versionId)) continue
		versionDataCache.value.set(versionId, { files: new Map(), loading: true })
		try {
			// TODO: switch to api-client once truman's vers stuff is merged
			const version = (await useBaseFetch(`version/${versionId}`)) as {
				files: Array<{
					filename: string
					file_name?: string
					hashes: { sha512: string; sha1: string }
				}>
			}
			const filesMap = new Map<string, string>()
			for (const file of version.files) {
				const name = file.file_name ?? file.filename
				filesMap.set(name, file.hashes.sha512)
			}
			versionDataCache.value.set(versionId, { files: filesMap, loading: false })
		} catch (error) {
			console.error(`Failed to fetch version ${versionId}:`, error)
			versionDataCache.value.set(versionId, {
				files: new Map(),
				loading: false,
				error: 'Failed',
			})
		}
	}
}

function getFileHash(versionId: string, fileName: string): string | undefined {
	return versionDataCache.value.get(versionId)?.files.get(fileName)
}

function isHashLoading(versionId: string): boolean {
	return versionDataCache.value.get(versionId)?.loading ?? false
}

function show() {
	const versionIds = props.unsafeFiles.map((f) => f.file.version_id)
	fetchVersionHashes(versionIds)
	modalRef.value?.show()
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
					</tr>
				</thead>
				<tbody>
					<tr v-for="item in unsafeFiles" :key="item.file.file_id">
						<td class="py-1 pr-2">
							<LoaderCircleIcon
								v-if="isHashLoading(item.file.version_id)"
								class="size-4 animate-spin text-secondary"
							/>
							<ButtonStyled
								v-else-if="getFileHash(item.file.version_id, item.file.file_name)"
								size="small"
								type="standard"
							>
								<button @click="copy(getFileHash(item.file.version_id, item.file.file_name)!)">
									<ClipboardCopyIcon class="size-4" />
									Copy
								</button>
							</ButtonStyled>
							<span v-else class="text-tertiary italic">N/A</span>
						</td>
						<td class="py-1 pr-2">
							<CopyCode :text="item.file.version_id" />
						</td>
						<td class="py-1 pr-2">
							<CopyCode :text="item.file.file_name" />
						</td>
						<td class="py-1">
							<ButtonStyled size="small" type="standard">
								<button @click="copy(item.file.download_url)">
									<ClipboardCopyIcon class="size-4" />
									Copy
								</button>
							</ButtonStyled>
						</td>
					</tr>
				</tbody>
			</table>

			<p v-else class="text-sm italic text-secondary">No files currently marked as malicious.</p>

			<div class="flex justify-end">
				<ButtonStyled>
					<button @click="hide">
						<XIcon class="size-4" />
						Close
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>
