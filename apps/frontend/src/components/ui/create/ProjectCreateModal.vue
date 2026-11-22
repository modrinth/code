<template>
	<NewModal
		ref="modal"
		:header="
			projectType === 'server'
				? formatMessage(messages.serverProjectTitle)
				: formatMessage(messages.title)
		"
		max-width="550px"
	>
		<div class="flex w-full flex-col gap-6">
			<RestrictedActionAlert v-model="isRestricted" scope="PROJECT_CREATE" @navigate="cancel" />
			<CreateLimitAlert v-model="hasHitLimit" type="project" />

			<Field :label="formatMessage(messages.nameLabel)" input-id="name">
				<Input
					id="name"
					v-model="name"
					:maxlength="64"
					:placeholder="formatMessage(messages.namePlaceholder)"
					autocomplete="off"
					wrapper-class="w-full"
					:disabled="hasHitLimit"
					@update:model-value="updatedName()"
				/>
			</Field>
			<Field :label="formatMessage(messages.summaryLabel)" input-id="additional-information">
				<Textarea
					id="additional-information"
					v-model="description"
					:maxlength="256"
					:placeholder="formatMessage(messages.summaryPlaceholder)"
					:disabled="hasHitLimit"
				/>
				<template #bottom>
					<p class="mb-0 mt-2 text-sm text-secondary">{{ formatMessage(projectSummaryHelp) }}</p>
				</template>
			</Field>
			<Field
				:label="formatMessage(messages.urlLabel)"
				input-id="slug"
				@focusin="onSlugSuggestionFocusIn"
				@focusout="onSlugSuggestionFocusOut"
			>
				<Input
					id="slug"
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
			<Field :label="formatMessage(messages.ownerLabel)" input-id="owner">
				<Combobox
					v-model="owner"
					search-name="owner"
					:search-input-attrs="{ id: 'owner' }"
					:options="[userOption, ...ownerOptions]"
					searchable
					:show-chevron="false"
					:disabled="hasHitLimit"
					select-search-text-on-focus
					show-icon-in-selected
				/>
			</Field>
			<Field :label="formatMessage(commonMessages.visibilityLabel)">
				<template #default="{ labelId }">
					<Chips
						v-model="visibility"
						:items="visibilityOptions"
						:format-label="(option) => formatMessage(visibilityLabels[option])"
						:format-tooltip="(option) => formatMessage(visibilityDescriptions[option])"
						:aria-labelledby="labelId"
						:aria-describedby="`${visibilityId}-description`"
						:capitalize="false"
						:disabled-items="hasHitLimit ? visibilityOptions : undefined"
					/>
				</template>
				<template #bottom>
					<p :id="`${visibilityId}-description`" class="mb-0 mt-2 text-sm text-secondary">
						{{ formatMessage(projectVisibilityApprovalHelp) }}
					</p>
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
import { OrganizationIcon, PlusIcon, XIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import {
	Chips,
	Combobox,
	type ComboboxOption,
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
import { computed, defineAsyncComponent, h, useId } from 'vue'

import SlugSuggestions from '~/components/ui/SlugSuggestions.vue'
import {
	generateUrlSlug,
	useProjectSlugSuggestions,
	useSlugSuggestionVisibility,
} from '~/composables/project-slug-suggestions'
import { projectSummaryHelp, projectVisibilityApprovalHelp } from '~/utils/project-messages'

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
	visibilityPublic: {
		id: 'create.project.visibility-public',
		defaultMessage: 'Public',
	},
	visibilityPublicDescription: {
		id: 'create.project.visibility-public-description',
		defaultMessage: 'Visible via URL, on your profile, and in search.',
	},
	visibilityUnlisted: {
		id: 'create.project.visibility-unlisted',
		defaultMessage: 'Unlisted',
	},
	visibilityUnlistedDescription: {
		id: 'create.project.visibility-unlisted-description',
		defaultMessage: 'Visible via URL only. Not shown on your profile or in search.',
	},
	visibilityPrivate: {
		id: 'create.project.visibility-private',
		defaultMessage: 'Private',
	},
	visibilityPrivateDescription: {
		id: 'create.project.visibility-private-description',
		defaultMessage: 'Not publicly visible. Only accessible to project members.',
	},
	missingFieldsTooltip: {
		id: 'create.project.missing-fields-tooltip',
		defaultMessage: 'Missing fields: {fields}',
	},
	unknownUser: {
		id: 'create.project.unknown-user',
		defaultMessage: 'Unknown user',
	},
	userAvatarAlt: {
		id: 'create.project.user-avatar-alt',
		defaultMessage: 'User Avatar',
	},
	organizationIconAlt: {
		id: 'create.project.organization-icon-alt',
		defaultMessage: '{name} Icon',
	},
})

const props = defineProps<{
	organizationId?: string | null
}>()

const modal = ref<InstanceType<typeof NewModal>>()
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
const ownerOptions = ref<ComboboxOption<string>[]>([])
const owner = ref<string | undefined>('self')
const organizations = ref<Labrinth.Projects.v3.Organization[]>([])
const visibilityId = useId()
const visibilityLabels = {
	approved: messages.visibilityPublic,
	unlisted: messages.visibilityUnlisted,
	private: messages.visibilityPrivate,
}
const visibilityDescriptions = {
	approved: messages.visibilityPublicDescription,
	unlisted: messages.visibilityUnlistedDescription,
	private: messages.visibilityPrivateDescription,
}
type Visibility = keyof typeof visibilityLabels
const visibilityOptions: Visibility[] = ['approved', 'unlisted', 'private']
const visibility = ref<Visibility>('approved')

const disableCreate = computed(() => {
	if (hasHitLimit.value || isRestricted.value) return true
	if (!name.value.trim() || !slug.value.trim()) return true
	if (!manualSlug.value && checkingSlugSuggestions.value) return true
	if (!manualSlug.value && !slugSuggestions.value.includes(slug.value)) return true
	if (description.value.trim().length < 3) return true
	if (owner.value !== 'self' && !organizations.value.find((org) => org.id === owner.value))
		return true
	return false
})

const missingFieldsTooltip = computed(() => {
	const missingFields = []
	if (!name.value.trim()) missingFields.push(formatMessage(messages.nameLabel))
	if (!slug.value.trim()) missingFields.push(formatMessage(messages.urlLabel))
	if (description.value.trim().length < 3) missingFields.push(formatMessage(messages.summaryLabel))
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

const userOption = computed(() => ({
	value: 'self',
	label: auth.value.user?.username || formatMessage(messages.unknownUser),
	icon: auth.value.user?.avatar_url
		? markRaw(
				defineAsyncComponent(() =>
					Promise.resolve({
						setup: () => () =>
							h('img', {
								src: auth.value.user?.avatar_url,
								alt: formatMessage(messages.userAvatarAlt),
								class: 'h-5 w-5 rounded-full',
							}),
					}),
				),
			)
		: undefined,
}))

const { labrinth } = injectModrinthClient()

async function fetchOrganizations() {
	if (!auth.value.user?.id) return

	try {
		const orgs = (await useBaseFetch(`user/${auth.value.user.id}/organizations`, {
			apiVersion: 3,
		})) as Labrinth.Projects.v3.Organization[]

		organizations.value = orgs || []

		ownerOptions.value = organizations.value.map((org) => ({
			value: org.id,
			label: org.name,
			icon: org.icon_url
				? markRaw(
						defineAsyncComponent(() =>
							Promise.resolve({
								setup: () => () =>
									h('img', {
										src: org.icon_url,
										alt: formatMessage(messages.organizationIconAlt, { name: org.name }),
										class: 'h-5 w-5 rounded',
									}),
							}),
						),
					)
				: markRaw(
						defineAsyncComponent(() =>
							Promise.resolve({
								setup: () => () => h(OrganizationIcon, { class: 'size-5' }),
							}),
						),
					),
		}))
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
