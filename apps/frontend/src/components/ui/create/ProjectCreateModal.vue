<template>
	<NewModal
		ref="modal"
		:header="
			projectType === 'server'
				? formatMessage(messages.serverProjectTitle)
				: formatMessage(messages.title)
		"
	>
		<div class="min-w-md flex max-w-md flex-col gap-6">
			<CreateLimitAlert v-model="hasHitLimit" type="project" />

			<div class="flex flex-col gap-2.5">
				<label for="name">
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.nameLabel) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input
					id="name"
					v-model="name"
					type="text"
					maxlength="64"
					:placeholder="formatMessage(messages.namePlaceholder)"
					autocomplete="off"
					:disabled="hasHitLimit"
					@input="updatedName()"
				/>
			</div>
			<label for="slug" class="flex flex-col gap-2.5">
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.urlLabel) }}
					<span class="text-brand-red">*</span>
				</span>
				<div class="text-input-wrapper">
					<div class="text-input-wrapper__before">https://modrinth.com/project/</div>
					<input
						id="slug"
						v-model="slug"
						class="w-full"
						type="text"
						maxlength="64"
						autocomplete="off"
						:disabled="hasHitLimit"
						@input="manualSlug = true"
					/>
				</div>
			</label>
			<div class="flex flex-col gap-2.5">
				<label for="owner">
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.ownerLabel) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<Combobox
					id="owner"
					v-model="owner"
					name="owner"
					:options="[userOption, ...ownerOptions]"
					searchable
					:disabled="hasHitLimit"
					show-icon-in-selected
				/>
				<span>{{ formatMessage(messages.ownerDescription) }}</span>
			</div>
			<div class="flex flex-col gap-2.5">
				<label for="visibility" class="flex flex-col gap-1">
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(commonMessages.visibilityLabel) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<Chips
					id="visibility"
					v-model="visibility"
					:items="visibilities"
					:format-label="(x) => x.display"
					:capitalize="false"
					:disabled="hasHitLimit"
				/>
				<span>{{ formatMessage(messages.visibilityDescription) }}</span>
			</div>
			<div class="flex flex-col gap-2.5">
				<label for="additional-information" class="flex flex-col gap-1">
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.summaryLabel) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<div class="textarea-wrapper">
					<textarea
						id="additional-information"
						v-model="description"
						maxlength="256"
						:placeholder="formatMessage(messages.summaryPlaceholder)"
						:disabled="hasHitLimit"
					/>
				</div>
				<span>{{ formatMessage(messages.summaryDescription) }}</span>
			</div>
			<div class="flex justify-end gap-2.5">
				<ButtonStyled class="w-24">
					<button @click="cancel">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand" class="w-32">
					<button :disabled="hasHitLimit" @click="createProject">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.createProject) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Chips,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { defineAsyncComponent, h } from 'vue'

import CreateLimitAlert from './CreateLimitAlert.vue'

type ProjectTypes = 'server' | 'project'
interface VisibilityOption {
	actual: string
	display: string
}
interface ShowOptions {
	type?: 'server' | 'project'
}

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()
const auth = (await useAuth()) as Ref<{
	user: { id: string; username: string; avatar_url: string } | null
}>

const messages = defineMessages({
	title: {
		id: 'create.project.title',
		defaultMessage: 'Creating a project',
	},
	serverProjectTitle: {
		id: 'create.project.server-project-title',
		defaultMessage: 'Creating a server project',
	},
	typeLabel: {
		id: 'create.project.type-label',
		defaultMessage: 'Type',
	},
	typeProject: {
		id: 'create.project.type-project',
		defaultMessage: 'Project',
	},
	typeServer: {
		id: 'create.project.type-server',
		defaultMessage: 'Server',
	},
	ownerLabel: {
		id: 'create.project.owner-label',
		defaultMessage: 'Owner',
	},
	ownerDescription: {
		id: 'create.project.owner-description',
		defaultMessage: 'Set the project owner as yourself, an individual, or an organization.',
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
	visibilityDescription: {
		id: 'create.project.visibility-description',
		defaultMessage: 'The visibility of your project after it has been approved.',
	},
	summaryLabel: {
		id: 'create.project.summary-label',
		defaultMessage: 'Summary',
	},
	summaryDescription: {
		id: 'create.project.summary-description',
		defaultMessage: 'A sentence or two that describes your project.',
	},
	summaryPlaceholder: {
		id: 'create.project.summary-placeholder',
		defaultMessage: 'This project adds...',
	},
	createProject: {
		id: 'create.project.create-project',
		defaultMessage: 'Create project',
	},
	createServerProject: {
		id: 'create.project.create-server-project',
		defaultMessage: 'Create server',
	},
	visibilityPublic: {
		id: 'create.project.visibility-public',
		defaultMessage: 'Public',
	},
	visibilityUnlisted: {
		id: 'create.project.visibility-unlisted',
		defaultMessage: 'Unlisted',
	},
	visibilityPrivate: {
		id: 'create.project.visibility-private',
		defaultMessage: 'Private',
	},
})

const props = defineProps<{
	organizationId?: string | null
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const hasHitLimit = ref(false)

const name = ref('')
const slug = ref('')
const description = ref('')
const manualSlug = ref(false)
const projectTypeOptions = ref<ComboboxOption<ProjectTypes>[]>([
	{
		value: 'project' as const,
		label: formatMessage(messages.typeProject),
	},
	{
		value: 'server' as const,
		label: formatMessage(messages.typeServer),
	},
])
const projectType = ref<ProjectTypes>('project')
const ownerOptions = ref<ComboboxOption<string>[]>([])
const owner = ref<string | null>('self')
const visibilities = ref<VisibilityOption[]>([
	{
		actual: 'approved',
		display: formatMessage(messages.visibilityPublic),
	},
	{
		actual: 'unlisted',
		display: formatMessage(messages.visibilityUnlisted),
	},
	{
		actual: 'private',
		display: formatMessage(messages.visibilityPrivate),
	},
])
const visibility = ref<VisibilityOption>(visibilities.value[0])

const cancel = () => {
	modal.value?.hide()
}

const userOption = {
	value: 'self',
	label: auth.value.user?.username || 'Unknown user',
	icon: auth.value.user?.avatar_url
		? defineAsyncComponent(() =>
				Promise.resolve({
					setup: () => () =>
						h('img', {
							src: auth.value.user?.avatar_url,
							alt: 'User Avatar',
							class: 'h-5 w-5 rounded',
						}),
				}),
			)
		: undefined,
}

async function createProject() {
	startLoading()

	const formData = new FormData()

	const projectData: Record<string, unknown> = {
		title: name.value.trim(),
		project_type: 'mod',
		slug: slug.value,
		description: description.value.trim(),
		body: '',
		requested_status: visibility.value.actual,
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
	}

	if (props.organizationId) {
		projectData.organization_id = props.organizationId
	}

	formData.append('data', JSON.stringify(projectData))

	try {
		await useBaseFetch('project', {
			method: 'POST',
			body: formData,
			headers: {
				'Content-Disposition': formData as unknown as string,
			},
		})

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

function show(event?: MouseEvent, options?: ShowOptions) {
	name.value = ''
	slug.value = ''
	description.value = ''
	manualSlug.value = false
	projectType.value = options?.type ?? 'project'
	modal.value?.show(event)
}

defineExpose({
	show,
})

function updatedName() {
	if (!manualSlug.value) {
		slug.value = name.value
			.trim()
			.toLowerCase()
			.replaceAll(' ', '-')
			.replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, '')
			.replaceAll(/--+/gm, '-')
	}
}
</script>
