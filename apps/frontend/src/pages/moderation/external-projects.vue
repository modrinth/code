<template>
	<NewModal ref="editModal" header="Edit external project">
		<form class="flex flex-col gap-2" @submit.prevent="saveExternalProjectEdit">
			<label class="font-semibold text-contrast" for="edit-form-title">Title</label>
			<StyledInput id="edit-form-title" v-model="editForm.title" type="text" />
			<label class="mt-2 font-semibold text-contrast" for="edit-form-link">Link</label>
			<StyledInput id="edit-form-link" v-model="editForm.link" type="text" />
			<label class="mt-2 font-semibold text-contrast" for="edit-form-cf-id">
				CurseForge project ID
			</label>
			<StyledInput id="edit-form-cf-id" v-model="editForm.flameProjectId" type="text" />
			<label class="mt-2 font-semibold text-contrast" for="edit-form-status">Allowed?</label>
			<Combobox
				id="edit-form-status"
				v-model="editForm.status"
				:options="statusOptions"
				class="!w-full"
			/>
			<label class="mt-2 font-semibold text-contrast" for="edit-form-proof">Proof</label>
			<StyledInput
				id="edit-form-proof"
				v-model="editForm.proof"
				type="text"
				multiline
				resize="both"
				class="w-[30rem]"
			/>
			<label class="mt-2 font-semibold text-contrast" for="edit-form-exceptions">
				Exceptions / notes
			</label>
			<StyledInput
				id="edit-form-exceptions"
				v-model="editForm.exceptions"
				type="text"
				multiline
				resize="both"
				class="w-[30rem]"
			/>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button @click="closeEditModal">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="submit" :disabled="isSavingEdit">
						{{ isSavingEdit ? 'Saving...' : 'Save' }}
					</button>
				</ButtonStyled>
			</div>
		</form>
	</NewModal>
	<div>
		<form class="flex gap-2" @submit.prevent="executeSearch">
			<StyledInput
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				placeholder="Search external projects..."
				clearable
				wrapper-class="flex-1 w-full"
			/>
			<ButtonStyled color="brand">
				<button type="submit">
					<SearchIcon aria-hidden="true" />
					Search by title
				</button>
			</ButtonStyled>
			<ButtonStyled>
				<button type="button" @click="executeFlameIdLookup">
					<BinaryIcon aria-hidden="true" />
					Lookup CurseForge ID
				</button>
			</ButtonStyled>
			<ButtonStyled>
				<button type="button" @click="executeSha1Lookup">
					<HashIcon aria-hidden="true" />
					Lookup SHA-1
				</button>
			</ButtonStyled>
		</form>
		<div>
			<template v-if="lastSearchKind !== 'none'">
				<template v-if="lookupNeedsInput">
					<EmptyState
						type="no-search-result"
						:heading="lookupEmptyHeading"
						:description="lookupEmptyDescription"
					/>
				</template>
				<template v-else>
					<EmptyState
						v-if="isLoading"
						type="no-search-result"
						heading="Loading external projects..."
					/>
					<div v-else-if="displayProjects.length > 0" class="mt-4 flex flex-col gap-3">
						<ExternalProjectLookupCard
							v-for="project in displayProjects"
							:key="project.id"
							:title="project.title"
							:state="project.status"
							:link="project.link"
							:notes="project.exceptions"
							:proof="project.proof"
							:files="project.files"
							:cf_id="project.flame_project_id"
						>
							<template #actions>
								<ButtonStyled>
									<button @click="openEditModal(project)">
										<EditIcon />
										Edit
									</button>
								</ButtonStyled>
							</template>
						</ExternalProjectLookupCard>
					</div>
					<EmptyState v-else type="no-search-result" :heading="noResultsHeading" />
				</template>
			</template>
			<EmptyState
				v-else
				type="no-search-result"
				heading="Enter a search term to get started"
				description="Type at least 3 characters of a project's title to begin browsing."
			/>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { BinaryIcon, EditIcon, HashIcon, SearchIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	EmptyState,
	type ExternalLicenseStatus,
	externalProjectLicenseStatusMessages,
	ExternalProjectLookupCard,
	injectModrinthClient,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()

const query = ref('')
const isLoading = ref(false)
const isSavingEdit = ref(false)
const client = injectModrinthClient()
const editModal = useTemplateRef<InstanceType<typeof NewModal>>('editModal')

useHead({ title: 'External projects - Modrinth' })

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

type SearchKind = 'none' | 'title' | 'flame_id' | 'sha1'

const lastSearchKind = ref<SearchKind>('none')
const activeQuery = ref('')
const externalProjects = ref<ExternalProject[]>([])

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
		files: project.linked_files,
	}
}

const displayProjects = computed(() => {
	return externalProjects.value
})

const lookupNeedsInput = computed(() => {
	const q = activeQuery.value
	const kind = lastSearchKind.value

	if (kind === 'title') {
		return q.length < 3
	}
	if (kind === 'flame_id' || kind === 'sha1') {
		return q.trim().length === 0
	}
	return false
})

const lookupEmptyHeading = computed(() => {
	const kind = lastSearchKind.value
	if (kind === 'title') {
		return 'Enter a search term to get started'
	}
	if (kind === 'flame_id') {
		return 'Enter a CurseForge project ID'
	}
	if (kind === 'sha1') {
		return 'Enter a SHA-1 hash'
	}
	return ''
})

const lookupEmptyDescription = computed(() => {
	const kind = lastSearchKind.value
	if (kind === 'title') {
		return "Type at least 3 characters of a project's title to begin browsing."
	}
	if (kind === 'flame_id') {
		return 'Type the numeric project ID, then use lookup for an exact match.'
	}
	if (kind === 'sha1') {
		return 'Paste the full 40-character hex hash, then use lookup for an exact match.'
	}
	return ''
})

const noResultsHeading = computed(() => {
	const kind = lastSearchKind.value
	if (kind === 'title') {
		return 'No projects matched that title search'
	}
	if (kind === 'flame_id') {
		return 'No external project has that CurseForge ID'
	}
	if (kind === 'sha1') {
		return 'No external file has that SHA-1 hash'
	}
	return 'No projects matched that title search'
})

const statusOptions = computed<ComboboxOption<ExternalLicenseStatus>[]>(() => [
	{ value: 'yes' as const, label: formatMessage(externalProjectLicenseStatusMessages.yes) },
	{
		value: 'with-attribution-and-source' as const,
		label: formatMessage(externalProjectLicenseStatusMessages['with-attribution-and-source']),
	},
	{
		value: 'with-attribution' as const,
		label: formatMessage(externalProjectLicenseStatusMessages['with-attribution']),
	},
	{ value: 'no' as const, label: formatMessage(externalProjectLicenseStatusMessages.no) },
	{
		value: 'permanent-no' as const,
		label: formatMessage(externalProjectLicenseStatusMessages['permanent-no']),
	},
	{
		value: 'unidentified' as const,
		label: formatMessage(externalProjectLicenseStatusMessages.unidentified),
	},
])

const editingProjectId = ref<number | null>(null)
const editForm = ref({
	title: '',
	status: 'unidentified' as ExternalLicenseStatus,
	link: '',
	proof: '',
	exceptions: '',
	flameProjectId: '',
})

function openEditModal(project: ExternalProject) {
	editingProjectId.value = project.id
	editForm.value = {
		title: project.title ?? '',
		status: project.status,
		link: project.link ?? '',
		proof: project.proof ?? '',
		exceptions: project.exceptions ?? '',
		flameProjectId: project.flame_project_id?.toString() ?? '',
	}
	editModal.value?.show()
}

function closeEditModal() {
	editModal.value?.hide()
}

async function saveExternalProjectEdit() {
	if (!editingProjectId.value) return
	isSavingEdit.value = true

	const parsedFlameProjectId = Number.parseInt(editForm.value.flameProjectId.trim(), 10)

	try {
		const updated = await client.labrinth.external_projects_internal.update(
			editingProjectId.value,
			{
				status: editForm.value.status,
				title: editForm.value.title.trim() || undefined,
				link: editForm.value.link.trim() || undefined,
				proof: editForm.value.proof.trim() || undefined,
				exceptions: editForm.value.exceptions.trim() || undefined,
				flame_project_id: Number.isFinite(parsedFlameProjectId) ? parsedFlameProjectId : undefined,
			},
		)

		const mapped = mapExternalProject(updated)
		const index = externalProjects.value.findIndex((project) => project.id === mapped.id)
		if (index >= 0) {
			externalProjects.value[index] = mapped
		}

		closeEditModal()
	} catch (error) {
		console.error('Failed to update external project', error)
	} finally {
		isSavingEdit.value = false
	}
}

async function executeLookup(kind: SearchKind) {
	if (kind === 'none') return

	lastSearchKind.value = kind
	activeQuery.value = query.value
	externalProjects.value = []

	if (lookupNeedsInput.value) {
		return
	}

	isLoading.value = true

	try {
		if (kind === 'title') {
			const response = await client.labrinth.external_projects_internal.search({
				title: activeQuery.value.trim(),
			})
			externalProjects.value = response.map(mapExternalProject)
			return
		}

		if (kind === 'flame_id') {
			const parsedFlameId = Number.parseInt(activeQuery.value.trim(), 10)
			if (!Number.isFinite(parsedFlameId)) {
				externalProjects.value = []
				return
			}

			const response = await client.labrinth.external_projects_internal.search({
				flame_id: parsedFlameId,
			})
			externalProjects.value = response.map(mapExternalProject)
			return
		}

		if (kind === 'sha1') {
			const response = await client.labrinth.external_projects_internal.getBySha1(
				activeQuery.value.trim(),
			)
			externalProjects.value = [mapExternalProject(response)]
		}
	} catch (error) {
		console.error('Failed to query external projects', error)
		externalProjects.value = []
	} finally {
		isLoading.value = false
	}
}

async function executeSearch() {
	await executeLookup('title')
}

async function executeFlameIdLookup() {
	await executeLookup('flame_id')
}

async function executeSha1Lookup() {
	await executeLookup('sha1')
}
</script>
