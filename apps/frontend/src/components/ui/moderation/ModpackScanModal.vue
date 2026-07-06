<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { FolderSearchIcon, StarIcon } from '@modrinth/assets'
import { ButtonStyled, injectModrinthClient, NewModal } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

const props = defineProps<{
	project_id: string
}>()

const client = injectModrinthClient()
const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')

const files = ref<Labrinth.Versions.v2.VersionFile[]>([])
const scans = ref<Map<string, Labrinth.Attribution.Internal.FileScanResponse>>(new Map())
const isFetching = ref(false)

async function fetchAllVersions() {
	const versions = await client.labrinth.versions_v2.getProjectVersions(props.project_id)
	files.value = versions.flatMap((version) => version.files)
}

async function fetchAllScans() {
	isFetching.value = true
	for (const file of files.value) {
		scans.value.set(file.id!, await client.labrinth.attribution_internal.scanFile(file.id!))
	}
	isFetching.value = false
}

function show() {
	isFetching.value = false
	fetchAllVersions()
	modalRef.value?.show()
}

function hide() {
	modalRef.value?.hide()
}
defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modalRef"
		width="60vw"
		:close-on-click-outside="false"
		:close-on-esc="false"
		:disable-close="isFetching"
	>
		<template #title>
			<div class="flex w-full items-center justify-between gap-2">
				<span class="text-2xl font-semibold text-contrast">
					Modpack Scan ({{ scans.size }}/{{ files.length }} Files)
				</span>
				<div>
					<ButtonStyled circular>
						<button v-tooltip="'Scan All Files'" :disabled="isFetching" @click="fetchAllScans">
							<FolderSearchIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>

		<div class="w-full">
			<table class="w-full table-fixed border-collapse border-0">
				<thead>
					<tr class="bg-surface-5 font-bold">
						<td class="p-2" style="width: 80%">Pack File Name</td>
						<td class="p-2 text-center" style="width: 10%">New Files</td>
						<td class="p-2 text-center" style="width: 10%">New Groups</td>
					</tr>
				</thead>
				<tbody>
					<template v-for="(file, index) in files" :key="file.id">
						<tr :class="index % 2 === 0 ? 'bg-surface-1' : 'bg-surface-2'">
							<td class="flex h-full flex-row items-center gap-1 p-2">
								<StarIcon v-if="file.primary" />{{ file.filename }}
							</td>

							<td class="p-2 text-center">
								<span v-if="!scans.get(file.id!!)">...</span>
								<span v-else>{{ scans.get(file.id!!)?.new_attribution_files!!! }}</span>
							</td>

							<td class="p-2 text-center">
								<span v-if="!scans.get(file.id!!)">...</span>
								<span v-else>{{ scans.get(file.id!!)?.new_attribution_groups!!! }}</span>
							</td>
						</tr>
						<tr
							v-if="scans.get(file.id!!)?.scanned_file_names"
							:class="index % 2 === 0 ? 'bg-surface-1' : 'bg-surface-2'"
							class="w-full"
						>
							<td colspan="3" class="p-2 pt-0">
								<details>
									<summary>
										Override Files ({{ scans.get(file.id!!)?.scanned_file_names?.length || 0 }})
									</summary>
									<div class="flex flex-wrap gap-1 pt-2">
										<span
											v-for="name of scans.get(file.id!!)?.scanned_file_names || []"
											:key="name"
											v-tooltip="name"
											class="flex items-center gap-1 text-wrap rounded-full bg-button-bg px-2 py-0.5 text-xs font-medium text-contrast"
										>
											{{ name }}
										</span>
									</div>
								</details>
							</td>
						</tr>
					</template>
				</tbody>
			</table>
		</div>
	</NewModal>
</template>
