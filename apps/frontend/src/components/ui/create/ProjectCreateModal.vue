<template>
	<NewModal
		ref="modal"
		:header="
			projectType === 'server'
				? formatMessage(messages.serverProjectTitle)
				: formatMessage(messages.title)
		"
		max-width="550px"
		:initial-focus="getNameInput"
	>
		<div class="flex w-full flex-col gap-6">
			<RestrictedActionAlert v-model="isRestricted" scope="PROJECT_CREATE" @navigate="cancel" />
			<CreateLimitAlert v-model="hasHitLimit" type="project" />

			<Field :label="formatMessage(messages.nameLabel)" :input-id="nameId">
				<Input
					:id="nameId"
					ref="nameInput"
					v-model="name"
					:maxlength="64"
					:placeholder="formatMessage(messages.namePlaceholder)"
					autocomplete="off"
					wrapper-class="w-full"
					:disabled="hasHitLimit"
					@update:model-value="updatedName()"
				/>
			</Field>
			<Field
				:label="formatMessage(messages.urlLabel)"
				:input-id="slugId"
				@focusin="onSlugSuggestionFocusIn"
				@focusout="onSlugSuggestionFocusOut"
			>
				<Input
					:id="slugId"
					v-model="slug"
					:maxlength="64"
					class="w-full"
					type="text"
					autocomplete="off"
					:disabled="hasHitLimit"
					@update:model-value="manualSlug = true"
				>
					<template #prefix>https://modrinth.com/project/</template>
				</Input>
				<template #bottom>
					<SlugSuggestions
						:selected="slug"
						:suggestions="slugSuggestions"
						:visible="showSlugSuggestions"
						@select="selectSlugSuggestion"
					/>
				</template>
			</Field>
			<Field :label="formatMessage(messages.summaryLabel)" :input-id="summaryId">
				<Textarea
					:id="summaryId"
					v-model="description"
					:maxlength="256"
					:placeholder="formatMessage(messages.summaryPlaceholder)"
					:disabled="hasHitLimit"
				/>
				<template #subtext>
					{{ formatMessage(projectSummaryHelp) }}
				</template>
			</Field>
			<Field :label="formatMessage(messages.ownerLabel)" :input-id="ownerId">
				<ProjectOwnerCombobox
					v-model="owner"
					:input-id="ownerId"
					:options="ownerOptions"
					:disabled="hasHitLimit"
					hide-chevron
				/>
			</Field>
			<Field :label="formatMessage(commonMessages.visibilityLabel)">
				<template #default="{ labelId }">
					<Chips
						v-model="visibility"
						:items="visibilityOptions"
						:format-label="(option) => formatMessage(projectVisibilityOptions[option].label)"
						:format-tooltip="
							(option) => formatMessage(projectVisibilityOptions[option].description)
						"
						:aria-labelledby="labelId"
						:aria-describedby="visibility !== 'private' ? visibilityId : undefined"
						:capitalize="false"
						:disabled-items="hasHitLimit ? visibilityOptions : undefined"
					/>
				</template>
				<template v-if="visibility !== 'private'" #subtext>
					<span :id="visibilityId">
						{{
							formatMessage(projectMessages.visibilityApprovalHelp, { contentType: projectType })
						}}
					</span>
				</template>
			</Field>
			<div class="flex justify-end gap-2.5">
				<Button type="outlined" @click="cancel">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					v-tooltip="missingFieldsTooltip"
					type="colored"
					color="brand"
					:disabled="disableCreate"
					@click="createProject"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.createProject) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	Chips,
	commonMessages,
	defineMessages,
	Field,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	Textarea,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { computed, useId } from 'vue'

import ProjectOwnerCombobox from '~/components/ui/ProjectOwnerCombobox.vue'
import SlugSuggestions from '~/components/ui/SlugSuggestions.vue'
import {
	generateUrlSlug,
	useProjectSlugSuggestions,
	useSlugSuggestionVisibility,
} from '~/composables/project-slug-suggestions'
import {
	projectMessages,
	projectSummaryHelp,
	projectVisibilityOptions,
} from '~/utils/project-messages'

import CreateLimitAlert from './CreateLimitAlert.vue'
import RestrictedActionAlert from './RestrictedActionAlert.vue'

type ProjectType = 'server' | 'project'

interface ShowOptions {
	type?: ProjectType
}

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()
const debug = useDebugLogger('ProjectCreateModal')

defineExpose({
	show,
})

const auth = await useAuth()

const messages = defineMessages({
	title: {
		id: 'create.project.title',
		defaultMessage: 'Creating a project',
	},
	serverProjectTitle: {
		id: 'create.project.server-project-title',
		defaultMessage: 'Creating a server project',
	},
	ownerLabel: {
		id: 'create.project.owner-label',
		defaultMessage: 'Owner',
	},
	nameLabel: {
		id: 'create.project.name-label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'create.project.name-placeholder',
		defaultMessage: 'Enter project name...',
	},
	urlLabel: {
		id: 'create.project.url-label',
		defaultMessage: 'URL',
	},
	summaryLabel: {
		id: 'create.project.summary-label',
		defaultMessage: 'Summary',
	},
	summaryPlaceholder: {
		id: 'create.project.summary-placeholder',
		defaultMessage: 'This project adds...',
	},
	createProject: {
		id: 'create.project.create-project',
		defaultMessage: 'Create project',
	},
	missingFieldsTooltip: {
		id: 'create.project.missing-fields-tooltip',
		defaultMessage: 'Missing fields: {fields}',
	},
	unknownUser: {
		id: 'create.project.unknown-user',
		defaultMessage: 'Unknown user',
	},
})

const props = defineProps<{
	organizationId?: string | null
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const nameInput = ref<InstanceType<typeof Input>>()
const hasHitLimit = ref(false)
const isRestricted = ref(false)

const name = ref('')
const slug = ref('')
const description = ref('')
const manualSlug = ref(false)
const {
	onFocusIn: onSlugSuggestionFocusIn,
	onFocusOut: onSlugSuggestionFocusOut,
	visible: showSlugSuggestions,
} = useSlugSuggestionVisibility()
const { checking: checkingSlugSuggestions, suggestions: slugSuggestions } =
	useProjectSlugSuggestions({
		title: name,
		username: () => auth.value.user?.username,
	})
const projectType = ref<ProjectType>('project')
const owner = ref<string | undefined>('self')
const organizations = ref<Labrinth.Projects.v3.Organization[]>([])
const nameId = useId()
const slugId = useId()
const summaryId = useId()
const ownerId = useId()
const visibilityId = useId()
type Visibility = keyof typeof projectVisibilityOptions
const visibilityOptions = Object.keys(projectVisibilityOptions) as Visibility[]
const visibility = ref<Visibility>('approved')

const disableCreate = computed(() => {
	if (hasHitLimit.value || isRestricted.value) return true
	if (!name.value.trim() || !slug.value.trim()) return true
	if (!manualSlug.value && checkingSlugSuggestions.value) return true
	if (!manualSlug.value && !slugSuggestions.value.includes(slug.value)) return true
	if (owner.value !== 'self' && !organizations.value.find((org) => org.id === owner.value))
		return true
	return false
})

const missingFieldsTooltip = computed(() => {
	const missingFields = []
	if (!name.value.trim()) missingFields.push(formatMessage(messages.nameLabel))
	if (!slug.value.trim()) missingFields.push(formatMessage(messages.urlLabel))
	if (owner.value !== 'self' && !organizations.value.find((org) => org.id === owner.value))
		missingFields.push(formatMessage(messages.ownerLabel))

	if (missingFields.length === 0) return ''
	return formatMessage(messages.missingFieldsTooltip, {
		fields: missingFields.join(', '),
	})
})

const cancel = () => {
	modal.value?.hide()
}

const ownerOptions = computed(() => [
	{
		value: 'self',
		label: auth.value.user?.username || formatMessage(messages.unknownUser),
		avatar_url: auth.value.user?.avatar_url,
	},
	...organizations.value.map((organization) => ({
		value: organization.id,
		label: organization.name,
		avatar_url: organization.icon_url,
		isOrganization: true,
	})),
])

const { labrinth } = injectModrinthClient()

async function fetchOrganizations() {
	if (!auth.value.user?.id) return

	try {
		const orgs = (await useBaseFetch(`user/${auth.value.user.id}/organizations`, {
			apiVersion: 3,
		})) as Labrinth.Projects.v3.Organization[]

		organizations.value = orgs || []

		if (props.organizationId) owner.value = props.organizationId
	} catch (err) {
		console.error('Failed to fetch organizations:', err)
	}
}

async function createProject() {
	if (disableCreate.value) return
	startLoading()

	const formData = new FormData()

	const projectData: Labrinth.Projects.v2.CreateProjectBase = {
		title: name.value.trim(),
		project_type: 'mod',
		slug: slug.value,
		description: description.value.trim(),
		body: '',
		requested_status: visibility.value,
		initial_versions: [],
		team_members: [
			{
				user_id: auth.value.user?.id,
				name: auth.value.user?.username,
				role: 'Owner',
			},
		],
		categories: [],
		client_side: 'required',
		server_side: 'required',
		license_id: 'LicenseRef-Unknown',
		is_draft: true,
		organization_id: owner.value !== 'self' ? owner.value : undefined,
	}

	formData.append('data', JSON.stringify(projectData))

	try {
		let createdProjectId: string | undefined

		if (projectType.value === 'server') {
			const result = await labrinth.projects_v3.createServerProject({
				base: {
					name: projectData.title,
					slug: projectData.slug,
					summary: projectData.description,
					description: '',
					requested_status: projectData.requested_status,
					organization_id: owner.value !== 'self' ? owner.value : undefined,
				},
				minecraft_server: {
					// empty component
				},
				minecraft_java_server: {
					address: '',
				},
				minecraft_bedrock_server: {
					address: '',
				},
			})
			createdProjectId = result.id
		} else {
			const result = (await useBaseFetch('project', {
				method: 'POST',
				body: formData,
				headers: {
					'Content-Disposition': formData as unknown as string,
				},
			})) as Labrinth.Projects.v3.Project
			createdProjectId = result.id
			debug(createdProjectId)
		}

		modal.value?.hide()
		await router.push(`/project/${slug.value}/settings`)
	} catch (err: unknown) {
		const error = err as { data?: { description?: string } }
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: error.data?.description ?? String(err),
			type: 'error',
		})
	}
	stopLoading()
}

async function show(event?: MouseEvent, options?: ShowOptions) {
	name.value = ''
	slug.value = ''
	description.value = ''
	manualSlug.value = false
	showSlugSuggestions.value = false
	owner.value = 'self'
	projectType.value = options?.type ?? 'project'
	await fetchOrganizations()
	modal.value?.show(event)
}

function getNameInput() {
	const root: HTMLElement | undefined = nameInput.value?.$el
	return root?.querySelector<HTMLInputElement>('input:enabled') ?? null
}

function updatedName() {
	if (!manualSlug.value) {
		slug.value = generateUrlSlug(name.value)
	}
}

function selectSlugSuggestion(suggestion: string) {
	slug.value = suggestion
	manualSlug.value = true
}

watch([slugSuggestions, checkingSlugSuggestions], ([suggestions, checking]) => {
	if (!manualSlug.value && !checking) slug.value = suggestions[0] ?? ''
})
</script>
