<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)">
		<div class="min-w-md flex max-w-md flex-col gap-3">
			<CreateLimitAlert v-model="hasHitLimit" type="org" />
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
					@input="updateSlug"
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
					<div class="text-input-wrapper__before">https://modrinth.com/organization/</div>
					<input
						id="slug"
						v-model="slug"
						type="text"
						maxlength="64"
						autocomplete="off"
						:disabled="hasHitLimit"
						@input="setManualSlug"
					/>
				</div>
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
			<p class="m-0">
				{{ formatMessage(messages.ownershipInfo) }}
			</p>
			<div class="flex justify-end gap-2">
				<ButtonStyled class="w-24">
					<button @click="hide">
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.cancel) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand" class="w-40">
					<button :disabled="hasHitLimit" @click="createOrganization">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.createOrganization) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager, NewModal } from '@modrinth/ui'
import { defineMessages } from '@vintl/vintl'
import { ref } from 'vue'

import CreateLimitAlert from './CreateLimitAlert.vue'

const router = useNativeRouter()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'create.organization.title',
		defaultMessage: 'Creating an organization',
	},
	nameLabel: {
		id: 'create.organization.name-label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'create.organization.name-placeholder',
		defaultMessage: 'Enter organization name...',
	},
	urlLabel: {
		id: 'create.organization.url-label',
		defaultMessage: 'URL',
	},
	summaryLabel: {
		id: 'create.organization.summary-label',
		defaultMessage: 'Summary',
	},
	summaryDescription: {
		id: 'create.organization.summary-description',
		defaultMessage: 'A sentence or two that describes your organization.',
	},
	summaryPlaceholder: {
		id: 'create.organization.summary-placeholder',
		defaultMessage: 'An organization for...',
	},
	ownershipInfo: {
		id: 'create.organization.ownership-info',
		defaultMessage:
			'You will be the owner of this organization, but you can invite other members and transfer ownership at any time.',
	},
	cancel: {
		id: 'create.organization.cancel',
		defaultMessage: 'Cancel',
	},
	createOrganization: {
		id: 'create.organization.create-organization',
		defaultMessage: 'Create organization',
	},
	errorTitle: {
		id: 'create.organization.error-title',
		defaultMessage: 'An error occurred',
	},
})

const name = ref<string>('')
const slug = ref<string>('')
const description = ref<string>('')
const manualSlug = ref<boolean>(false)
const hasHitLimit = ref<boolean>(false)
const modal = ref<InstanceType<typeof NewModal>>()

async function createOrganization(): Promise<void> {
	startLoading()
	try {
		const value = {
			name: name.value.trim(),
			description: description.value.trim(),
			slug: slug.value.trim().replace(/ +/g, ''),
		}

		const result: any = await useBaseFetch('organization', {
			method: 'POST',
			body: JSON.stringify(value),
			apiVersion: 3,
		})

		modal.value?.hide()

		await router.push(`/organization/${result.slug}`)
	} catch (err: any) {
		console.error(err)
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

function show(event?: MouseEvent): void {
	name.value = ''
	description.value = ''
	modal.value?.show(event)
}

function hide(): void {
	modal.value?.hide()
}

function updateSlug(): void {
	if (!manualSlug.value) {
		slug.value = name.value
			.trim()
			.toLowerCase()
			.replaceAll(' ', '-')
			.replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, '')
			.replaceAll(/--+/gm, '-')
	}
}

function setManualSlug(): void {
	manualSlug.value = true
}

defineExpose({
	show,
})
</script>

<style scoped lang="scss">
.modal-creation {
	input {
		width: 20rem;
		max-width: 100%;
	}

	.text-input-wrapper {
		width: 100%;
	}

	textarea {
		min-height: 5rem;
	}

	.input-group {
		margin-top: var(--gap-md);
	}
}
</style>
