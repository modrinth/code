<template>
	<div>
		<h2 v-if="modPackData" class="m-0 mb-2 text-lg font-extrabold">
			Modpack permissions ({{ Math.min(modPackData.length, currentIndex + 1) }} /
			{{ modPackData.length }})
		</h2>

		<div v-if="!modPackData">Loading data...</div>

		<div v-else-if="modPackData.length === 0">
			<p>All permissions already obtained.</p>
		</div>

		<div v-else-if="!modPackData[currentIndex]">
			<p>All permission checks complete!</p>
		</div>

		<div v-else>
			<div v-if="modPackData[currentIndex].type === 'unknown'">
				<p>What is the approval type of {{ modPackData[currentIndex].file_name }}?</p>
				<div class="input-group">
					<ButtonStyled
						v-for="(option, index) in fileApprovalTypes"
						:key="index"
						:color="modPackData[currentIndex].status === option.id ? 'brand' : 'standard'"
						@click="setStatus(currentIndex, option.id)"
					>
						<button>
							{{ option.name }}
						</button>
					</ButtonStyled>
				</div>
				<div v-if="modPackData[currentIndex].status !== 'unidentified'" class="flex flex-col gap-1">
					<label for="proof">
						<span class="label__title">Proof</span>
					</label>
					<input
						id="proof"
						v-model="(modPackData[currentIndex] as ModerationUnknownModpackItem).proof"
						type="text"
						autocomplete="off"
						placeholder="Enter proof of status..."
						@input="persistAll()"
					/>
					<label for="link">
						<span class="label__title">Link</span>
					</label>
					<input
						id="link"
						v-model="(modPackData[currentIndex] as ModerationUnknownModpackItem).url"
						type="text"
						autocomplete="off"
						placeholder="Enter link of project..."
						@input="persistAll()"
					/>
					<label for="title">
						<span class="label__title">Title</span>
					</label>
					<input
						id="title"
						v-model="(modPackData[currentIndex] as ModerationUnknownModpackItem).title"
						type="text"
						autocomplete="off"
						placeholder="Enter title of project..."
						@input="persistAll()"
					/>
				</div>
			</div>

			<div v-else-if="modPackData[currentIndex].type === 'flame'">
				<p>
					What is the approval type of {{ modPackData[currentIndex].title }} (<a
						:href="modPackData[currentIndex].url"
						target="_blank"
						class="text-link"
						>{{ modPackData[currentIndex].url }}</a
					>)?
				</p>
				<div class="input-group">
					<ButtonStyled
						v-for="(option, index) in fileApprovalTypes"
						:key="index"
						:color="modPackData[currentIndex].status === option.id ? 'brand' : 'standard'"
						@click="setStatus(currentIndex, option.id)"
					>
						<button>
							{{ option.name }}
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div
				v-if="
					['unidentified', 'no', 'with-attribution'].includes(
						modPackData[currentIndex].status || '',
					)
				"
			>
				<p v-if="modPackData[currentIndex].status === 'unidentified'">
					Does this project provide identification and permission for
					<strong>{{ modPackData[currentIndex].file_name }}</strong
					>?
				</p>
				<p v-else-if="modPackData[currentIndex].status === 'with-attribution'">
					Does this project provide attribution for
					<strong>{{ modPackData[currentIndex].file_name }}</strong
					>?
				</p>
				<p v-else>
					Does this project provide proof of permission for
					<strong>{{ modPackData[currentIndex].file_name }}</strong
					>?
				</p>
				<div class="input-group">
					<ButtonStyled
						v-for="(option, index) in filePermissionTypes"
						:key="index"
						:color="modPackData[currentIndex].approved === option.id ? 'brand' : 'standard'"
						@click="setApproval(currentIndex, option.id)"
					>
						<button>
							{{ option.name }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<div class="mt-4 flex gap-2">
			<ButtonStyled>
				<button :disabled="currentIndex <= 0" @click="goToPrevious">
					<LeftArrowIcon aria-hidden="true" />
					Previous
				</button>
			</ButtonStyled>
			<ButtonStyled v-if="modPackData && currentIndex < modPackData.length" color="blue">
				<button :disabled="!canGoNext" @click="goToNext">
					<RightArrowIcon aria-hidden="true" />
					{{ currentIndex + 1 >= modPackData.length ? 'Complete' : 'Next' }}
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import type {
	ModerationFlameModpackItem,
	ModerationJudgements,
	ModerationModpackItem,
	ModerationModpackPermissionApprovalType,
	ModerationModpackResponse,
	ModerationPermissionType,
	ModerationUnknownModpackItem,
} from '@modrinth/utils'
import { useLocalStorage, useSessionStorage } from '@vueuse/core'
import { computed, onMounted, ref, watch } from 'vue'

const props = defineProps<{
	projectId: string
	modelValue?: ModerationJudgements
}>()

const emit = defineEmits<{
	complete: []
	'update:modelValue': [judgements: ModerationJudgements]
}>()

const persistedModPackData = useLocalStorage<ModerationModpackItem[] | null>(
	`modpack-permissions-${props.projectId}`,
	null,
	{
		serializer: {
			read: (v: any) => (v ? JSON.parse(v) : null),
			write: (v: any) => JSON.stringify(v),
		},
	},
)

const persistedIndex = useLocalStorage<number>(`modpack-permissions-index-${props.projectId}`, 0)

const modPackData = useSessionStorage<ModerationModpackItem[] | null>(
	`modpack-permissions-data-${props.projectId}`,
	null,
	{
		serializer: {
			read: (v: any) => (v ? JSON.parse(v) : null),
			write: (v: any) => JSON.stringify(v),
		},
	},
)
const permanentNoFiles = useSessionStorage<ModerationModpackItem[]>(
	`modpack-permissions-permanent-no-${props.projectId}`,
	[],
	{
		serializer: {
			read: (v: any) => (v ? JSON.parse(v) : []),
			write: (v: any) => JSON.stringify(v),
		},
	},
)
const currentIndex = ref(0)

const fileApprovalTypes: ModerationModpackPermissionApprovalType[] = [
	{
		id: 'yes',
		name: 'Yes',
	},
	{
		id: 'with-attribution-and-source',
		name: 'With attribution and source',
	},
	{
		id: 'with-attribution',
		name: 'With attribution',
	},
	{
		id: 'no',
		name: 'No',
	},
	{
		id: 'permanent-no',
		name: 'Permanent no',
	},
	{
		id: 'unidentified',
		name: 'Unidentified',
	},
]

const filePermissionTypes: ModerationPermissionType[] = [
	{ id: 'yes', name: 'Yes' },
	{ id: 'no', name: 'No' },
]

function persistAll() {
	persistedModPackData.value = modPackData.value
	persistedIndex.value = currentIndex.value
}

watch(
	modPackData,
	(newValue) => {
		persistedModPackData.value = newValue
	},
	{ deep: true },
)

watch(currentIndex, (newValue) => {
	persistedIndex.value = newValue
})

function loadPersistedData(): void {
	if (persistedModPackData.value) {
		modPackData.value = persistedModPackData.value
	}
	currentIndex.value = persistedIndex.value
}

function clearPersistedData(): void {
	persistedModPackData.value = null
	persistedIndex.value = 0
}

async function fetchModPackData(): Promise<void> {
	try {
		const data = (await useBaseFetch(`moderation/project/${props.projectId}`, {
			internal: true,
		})) as ModerationModpackResponse

		const permanentNoItems: ModerationModpackItem[] = Object.entries(data.identified || {})
			.filter(([_, file]) => file.status === 'permanent-no')
			.map(
				([sha1, file]): ModerationModpackItem => ({
					sha1,
					file_name: file.file_name,
					type: 'identified',
					status: file.status,
					approved: null,
				}),
			)
			.sort((a, b) => a.file_name.localeCompare(b.file_name))

		permanentNoFiles.value = permanentNoItems

		const sortedData: ModerationModpackItem[] = [
			...Object.entries(data.identified || {})
				.filter(
					([_, file]) =>
						file.status !== 'yes' &&
						file.status !== 'with-attribution-and-source' &&
						file.status !== 'permanent-no',
				)
				.map(
					([sha1, file]): ModerationModpackItem => ({
						sha1,
						file_name: file.file_name,
						type: 'identified',
						status: file.status,
						approved: null,
						...(file.status === 'unidentified' && {
							proof: '',
							url: '',
							title: '',
						}),
					}),
				)
				.sort((a, b) => a.file_name.localeCompare(b.file_name)),
			...Object.entries(data.unknown_files || {})
				.map(
					([sha1, fileName]): ModerationUnknownModpackItem => ({
						sha1,
						file_name: fileName,
						type: 'unknown',
						status: null,
						approved: null,
						proof: '',
						url: '',
						title: '',
					}),
				)
				.sort((a, b) => a.file_name.localeCompare(b.file_name)),
			...Object.entries(data.flame_files || {})
				.map(
					([sha1, info]): ModerationFlameModpackItem => ({
						sha1,
						file_name: info.file_name,
						type: 'flame',
						status: null,
						approved: null,
						id: info.id,
						title: info.title || info.file_name,
						url: info.url || `https://www.curseforge.com/minecraft/mc-mods/${info.id}`,
					}),
				)
				.sort((a, b) => a.file_name.localeCompare(b.file_name)),
		]

		if (modPackData.value) {
			const existingMap = new Map(modPackData.value.map((item) => [item.sha1, item]))

			sortedData.forEach((item) => {
				const existing = existingMap.get(item.sha1)
				if (existing) {
					Object.assign(item, {
						status: existing.status,
						approved: existing.approved,
						...(item.type === 'unknown' && {
							proof: (existing as ModerationUnknownModpackItem).proof || '',
							url: (existing as ModerationUnknownModpackItem).url || '',
							title: (existing as ModerationUnknownModpackItem).title || '',
						}),
						...(item.type === 'flame' && {
							url: (existing as ModerationFlameModpackItem).url || item.url,
							title: (existing as ModerationFlameModpackItem).title || item.title,
						}),
					})
				}
			})
		}

		modPackData.value = sortedData
		persistAll()
	} catch (error) {
		console.error('Failed to fetch modpack data:', error)
		modPackData.value = []
		permanentNoFiles.value = []
		persistAll()
	}
}

function goToPrevious(): void {
	if (currentIndex.value > 0) {
		currentIndex.value--
		persistAll()
	}
}

watch(
	modPackData,
	(newValue) => {
		persistedModPackData.value = newValue
	},
	{ deep: true },
)

function goToNext(): void {
	if (modPackData.value && currentIndex.value < modPackData.value.length) {
		currentIndex.value++

		if (currentIndex.value >= modPackData.value.length) {
			const judgements = getJudgements()
			emit('update:modelValue', judgements)
			emit('complete')
			clearPersistedData()
		} else {
			persistAll()
		}
	}
}

function setStatus(index: number, status: ModerationModpackPermissionApprovalType['id']): void {
	if (modPackData.value && modPackData.value[index]) {
		modPackData.value[index].status = status
		modPackData.value[index].approved = null
		persistAll()
		emit('update:modelValue', getJudgements())
	}
}

function setApproval(index: number, approved: ModerationPermissionType['id']): void {
	if (modPackData.value && modPackData.value[index]) {
		modPackData.value[index].approved = approved
		persistAll()
		emit('update:modelValue', getJudgements())
	}
}

const canGoNext = computed(() => {
	if (!modPackData.value || !modPackData.value[currentIndex.value]) return false
	const current = modPackData.value[currentIndex.value]
	return current.status !== null
})

function getJudgements(): ModerationJudgements {
	if (!modPackData.value) return {}

	const judgements: ModerationJudgements = {}

	modPackData.value.forEach((item) => {
		if (item.type === 'flame') {
			judgements[item.sha1] = {
				type: 'flame',
				id: item.id,
				status: item.status,
				link: item.url,
				title: item.title,
				file_name: item.file_name,
			}
		} else if (item.type === 'unknown') {
			judgements[item.sha1] = {
				type: 'unknown',
				status: item.status,
				proof: item.proof,
				link: item.url,
				title: item.title,
				file_name: item.file_name,
			}
		}
	})

	return judgements
}

onMounted(() => {
	loadPersistedData()
	if (!modPackData.value) {
		fetchModPackData()
	}
})

watch(
	modPackData,
	(newValue) => {
		if (newValue && newValue.length === 0) {
			emit('complete')
			clearPersistedData()
		}
	},
	{ immediate: true },
)

watch(
	() => props.projectId,
	() => {
		clearPersistedData()
		loadPersistedData()
		if (!modPackData.value) {
			fetchModPackData()
		}
	},
)

function getModpackFiles(): {
	interactive: ModerationModpackItem[]
	permanentNo: ModerationModpackItem[]
} {
	return {
		interactive: modPackData.value || [],
		permanentNo: permanentNoFiles.value,
	}
}

defineExpose({
	getModpackFiles,
})
</script>

<style scoped>
.input-group {
	display: flex;
	gap: 0.5rem;
	margin-top: 0.5rem;
	margin-bottom: 0.5rem;
}

.modpack-buttons {
	margin-top: 1rem;
}
</style>
