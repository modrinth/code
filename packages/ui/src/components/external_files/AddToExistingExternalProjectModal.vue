<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useMutation } from '@tanstack/vue-query'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

import { Accordion, ButtonStyled, NewModal, StyledInput } from '#ui/components'

import { injectModrinthClient, injectNotificationManager } from '../../providers'
import AttributionGroupFilePicker from './AttributionGroupFilePicker.vue'
import {
	MODERATOR_ATTRIBUTION_KIND_LABELS,
	moderatorAttributionGroupTitle,
	parseInitialAttribution,
} from './external-project-utils'
import ExternalProjectLookupCard from './ExternalProjectLookupCard.vue'
import type { ExternalLicenseStatus } from './types.ts'

const props = defineProps<{
	group: Labrinth.Attribution.Internal.AttributionGroup
}>()

const emit = defineEmits<{
	(e: 'success'): void
}>()

type ExternalProject = {
	id: number
	title: string | null
	status: ExternalLicenseStatus
	link: string | null
	exceptions: string | null
	proof: string | null
	flame_project_id: number | null
	files: {
		sha1: string
		name: string | null
	}[]
}

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const searchAccordionRef = useTemplateRef<InstanceType<typeof Accordion>>('searchAccordionRef')
const filesAccordionRef = useTemplateRef<InstanceType<typeof Accordion>>('filesAccordionRef')

const query = ref('')
const isLoading = ref(false)
const hasSearched = ref(false)
const activeQuery = ref('')
const externalProjects = ref<ExternalProject[]>([])
const selectedProjectId = ref<number | null>(null)
const selectedSha1s = ref<Set<string>>(new Set())

const hasMultipleFiles = computed(() => props.group.files.length > 1)

const groupPreviewTitle = computed(() => moderatorAttributionGroupTitle(props.group))

const attributionKindLabel = computed(() => {
	const attribution = parseInitialAttribution(props.group.attribution)
	if (!attribution) {
		return null
	}
	return `Attribution: ${MODERATOR_ATTRIBUTION_KIND_LABELS[attribution.kind]}`
})

function mapExternalProject(
	project: Labrinth.ExternalProjects.Internal.ExternalProject,
): ExternalProject {
	return {
		id: project.id,
		title: project.title,
		status: project.status,
		link: project.link,
		exceptions: project.exceptions,
		proof: project.proof,
		flame_project_id: project.flame_project_id,
		files: project.linked_files ?? [],
	}
}

const searchNeedsInput = computed(() => hasSearched.value && activeQuery.value.trim().length < 3)

const addFilesMutation = useMutation({
	mutationFn: async () => {
		if (selectedProjectId.value === null) {
			throw new Error('No external project selected')
		}

		const sha1s = [...selectedSha1s.value]
		await Promise.all(
			sha1s.map((sha1) =>
				client.labrinth.external_projects_internal.addFile({
					hash: sha1,
					license_id: selectedProjectId.value!,
				}),
			),
		)
	},
	onSuccess: async () => {
		addNotification({
			type: 'success',
			title: 'Files added to existing entry',
		})
		await nextTick()
		hide()
		emit('success')
	},
	onError: (error: Error) => {
		addNotification({
			type: 'error',
			title: 'Could not add files to existing entry',
			text: error.message,
		})
	},
})

const canSubmit = computed(
	() =>
		selectedProjectId.value !== null &&
		selectedSha1s.value.size > 0 &&
		!addFilesMutation.isPending.value,
)

function resetForm() {
	query.value = ''
	hasSearched.value = false
	activeQuery.value = ''
	externalProjects.value = []
	selectedProjectId.value = null
	selectedSha1s.value = new Set(props.group.files.map((file) => file.sha1))
}

function handleSearchPanelOpen() {
	filesAccordionRef.value?.close()
}

function handleFilesPanelOpen() {
	searchAccordionRef.value?.close()
}

function selectProject(projectId: number) {
	selectedProjectId.value = projectId
}

async function executeSearch() {
	hasSearched.value = true
	activeQuery.value = query.value
	externalProjects.value = []
	selectedProjectId.value = null

	if (activeQuery.value.trim().length < 3) {
		return
	}

	isLoading.value = true

	try {
		const response = await client.labrinth.external_projects_internal.search({
			title: activeQuery.value.trim(),
		})
		externalProjects.value = response.map(mapExternalProject)
	} catch {
		externalProjects.value = []
	} finally {
		isLoading.value = false
	}
}

function handleSubmit() {
	addFilesMutation.mutate()
}

function show(event?: MouseEvent) {
	resetForm()
	modalRef.value?.show(event)
}

function hide() {
	modalRef.value?.hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modalRef"
		header="Add to existing entry"
		max-width="720px"
		:disable-close="addFilesMutation.isPending.value"
	>
		<div class="flex flex-col gap-4 w-[650px]">
			<div
				class="rounded-xl bg-surface-3 border border-solid border-surface-5 p-3 flex flex-col gap-1 shrink-0"
			>
				<span class="font-semibold text-contrast">{{ groupPreviewTitle }}</span>
				<span v-if="attributionKindLabel" class="text-secondary text-sm">{{
					attributionKindLabel
				}}</span>
			</div>
			<Accordion
				ref="searchAccordionRef"
				class="w-full bg-surface-4 border border-solid border-surface-5 rounded-2xl overflow-clip"
				button-class="p-4 w-full border-b border-solid border-b-surface-5 bg-surface-2 -mb-px hover:brightness-[--hover-brightness] group"
				open-by-default
				@on-open="handleSearchPanelOpen"
			>
				<template #title>
					<span class="flex items-center gap-3 text-contrast group-active:scale-[0.98]">
						Select an external project
					</span>
				</template>
				<div class="flex flex-col bg-surface-4 min-h-0">
					<form
						class="flex flex-wrap gap-2 shrink-0 p-4 bg-surface-3 border-0 border-b border-solid border-surface-5"
						@submit.prevent="executeSearch"
					>
						<StyledInput
							v-model="query"
							:icon="SearchIcon"
							type="text"
							autocomplete="off"
							placeholder="Search external projects…"
							clearable
							wrapper-class="flex-1 min-w-[12rem]"
							:disabled="addFilesMutation.isPending.value"
						/>
						<ButtonStyled color="brand">
							<button type="submit" :disabled="addFilesMutation.isPending.value">
								<SearchIcon aria-hidden="true" />
								Search
							</button>
						</ButtonStyled>
					</form>
					<div
						class="flex flex-col min-h-0 max-h-[min(50vh,24rem)] overflow-y-auto overflow-x-hidden"
					>
						<div
							v-if="searchNeedsInput || isLoading || externalProjects.length === 0"
							class="p-8 flex flex-col gap-2 items-center justify-center"
						>
							<span class="text-contrast font-semibold">
								<template v-if="searchNeedsInput"> Enter a search term to get started </template>
								<template v-else-if="isLoading"> Loading external projects… </template>
								<template v-else> No projects matched that search </template>
							</span>
							<span class="text-secondary text-sm">
								<template v-if="searchNeedsInput">
									Type at least 3 characters of a project's title to begin browsing.
								</template>
								<template v-else-if="isLoading"> Loading external projects… </template>
								<template v-else> No projects matched that search </template>
							</span>
						</div>
						<div v-else-if="externalProjects.length > 0" class="flex flex-col gap-3">
							<ExternalProjectLookupCard
								v-for="project in externalProjects"
								:key="project.id"
								:title="project.title"
								:state="project.status"
								:link="project.link"
								:notes="project.exceptions"
								:proof="project.proof"
								:files="project.files"
								:cf_id="project.flame_project_id"
								class="mx-4 mt-3"
							>
								<template #actions>
									<ButtonStyled color="brand">
										<button
											type="button"
											:disabled="
												selectedProjectId === project.id || addFilesMutation.isPending.value
											"
											@click="selectProject(project.id)"
										>
											{{ selectedProjectId === project.id ? 'Selected' : 'Select' }}
										</button>
									</ButtonStyled>
								</template>
							</ExternalProjectLookupCard>
						</div>
					</div>
				</div>
			</Accordion>

			<Accordion
				v-if="hasMultipleFiles"
				ref="filesAccordionRef"
				class="w-full bg-surface-4 border border-solid border-surface-5 rounded-2xl overflow-clip"
				button-class="p-4 w-full border-b border-solid border-b-surface-5 bg-surface-2 -mb-px hover:brightness-[--hover-brightness] group"
				@on-open="handleFilesPanelOpen"
			>
				<template #title>
					<span class="flex items-center gap-3 text-contrast group-active:scale-[0.98]">
						Select files to add
					</span>
				</template>
				<AttributionGroupFilePicker
					v-model:selected-sha1s="selectedSha1s"
					:files="group.files"
					:disabled="addFilesMutation.isPending.value"
				/>
			</Accordion>
			<div class="flex justify-end gap-2 w-full">
				<ButtonStyled type="outlined">
					<button type="button" :disabled="addFilesMutation.isPending.value" @click="hide">
						<XIcon class="size-4 shrink-0" />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" :disabled="!canSubmit" @click="handleSubmit">
						<SpinnerIcon
							v-if="addFilesMutation.isPending.value"
							class="size-4 shrink-0 animate-spin"
						/>
						<PlusIcon v-else class="size-4 shrink-0" />
						Add files to entry
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>
