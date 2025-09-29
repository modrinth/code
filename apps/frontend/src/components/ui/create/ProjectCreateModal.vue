<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)">
		<div class="min-w-md flex max-w-md flex-col gap-3">
			<CreateLimitAlert v-model="hasHitLimit" type="project" />
			<div class="flex flex-col gap-2">
				<label for="name">
					<span class="text-lg font-semibold text-contrast">
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
			<div class="flex flex-col gap-2">
				<label for="slug">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.urlLabel) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<div class="text-input-wrapper">
					<div class="text-input-wrapper__before">https://modrinth.com/project/</div>
					<input
						id="slug"
						v-model="slug"
						type="text"
						maxlength="64"
						autocomplete="off"
						:disabled="hasHitLimit"
						@input="manualSlug = true"
					/>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<label for="visibility" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.visibilityLabel) }}
						<span class="text-brand-red">*</span>
					</span>
					<span>{{ formatMessage(messages.visibilityDescription) }}</span>
				</label>
				<Chips
					id="visibility"
					v-model="visibility"
					:items="visibilities"
					:format-label="(x) => x.display"
					:capitalize="false"
					:disabled="hasHitLimit"
				/>
			</div>
			<div class="flex flex-col gap-2">
				<label for="additional-information" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.summaryLabel) }}
						<span class="text-brand-red">*</span>
					</span>
					<span>{{ formatMessage(messages.summaryDescription) }}</span>
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
			</div>
			<div class="flex justify-end gap-2">
				<ButtonStyled class="w-24">
					<button @click="cancel">
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.cancel) }}
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

<script setup>
import { PlusIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, Chips, injectNotificationManager, NewModal } from '@modrinth/ui'
import { defineMessages } from '@vintl/vintl'

import CreateLimitAlert from './CreateLimitAlert.vue'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()

const messages = defineMessages({
	title: {
		id: 'create.project.title',
		defaultMessage: 'Creating a project',
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
	visibilityLabel: {
		id: 'create.project.visibility-label',
		defaultMessage: 'Visibility',
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
	cancel: {
		id: 'create.project.cancel',
		defaultMessage: 'Cancel',
	},
	createProject: {
		id: 'create.project.create-project',
		defaultMessage: 'Create project',
	},
	errorTitle: {
		id: 'create.project.error-title',
		defaultMessage: 'An error occurred',
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

const props = defineProps({
	organizationId: {
		type: String,
		required: false,
		default: null,
	},
})

const modal = ref()
const hasHitLimit = ref(false)

const name = ref('')
const slug = ref('')
const description = ref('')
const manualSlug = ref(false)
const visibilities = ref([
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
const visibility = ref(visibilities.value[0])

const cancel = () => {
	modal.value.hide()
}

async function createProject() {
	startLoading()

	const formData = new FormData()

	const auth = await useAuth()

	const projectData = {
		title: name.value.trim(),
		project_type: 'mod',
		slug: slug.value,
		description: description.value.trim(),
		body: '',
		requested_status: visibility.value.actual,
		initial_versions: [],
		team_members: [
			{
				user_id: auth.value.user.id,
				name: auth.value.user.username,
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
				'Content-Disposition': formData,
			},
		})

		modal.value.hide()
		await router.push({
			name: 'type-id',
			params: {
				type: 'project',
				id: slug.value,
			},
		})
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

function show(event) {
	name.value = ''
	slug.value = ''
	description.value = ''
	manualSlug.value = false
	modal.value.show(event)
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
