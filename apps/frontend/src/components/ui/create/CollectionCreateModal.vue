<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)">
		<div class="min-w-md flex max-w-md flex-col gap-3">
			<CreateLimitAlert v-model="hasHitLimit" type="collection" />
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
				/>
			</div>
			<div class="flex flex-col gap-2">
				<label for="additional-information" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.summaryLabel)
					}}</span>
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
				{{ formatMessage(messages.collectionInfo, { count: projectIds.length }) }}
			</p>
			<div class="flex justify-end gap-2">
				<ButtonStyled class="w-24">
					<button @click="modal.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.cancel) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand" class="w-36">
					<button :disabled="hasHitLimit" @click="create">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.createCollection) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>
<script setup>
import { PlusIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager, NewModal } from '@modrinth/ui'
import { defineMessages } from '@vintl/vintl'

import CreateLimitAlert from './CreateLimitAlert.vue'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useNativeRouter()

const messages = defineMessages({
	title: {
		id: 'create.collection.title',
		defaultMessage: 'Creating a collection',
	},
	nameLabel: {
		id: 'create.collection.name-label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'create.collection.name-placeholder',
		defaultMessage: 'Enter collection name...',
	},
	summaryLabel: {
		id: 'create.collection.summary-label',
		defaultMessage: 'Summary',
	},
	summaryDescription: {
		id: 'create.collection.summary-description',
		defaultMessage: 'A sentence or two that describes your collection.',
	},
	summaryPlaceholder: {
		id: 'create.collection.summary-placeholder',
		defaultMessage: 'This is a collection of...',
	},
	collectionInfo: {
		id: 'create.collection.collection-info',
		defaultMessage:
			'Your new collection will be created as a public collection with {count, plural, =0 {no projects} one {# project} other {# projects}}.',
	},
	cancel: {
		id: 'create.collection.cancel',
		defaultMessage: 'Cancel',
	},
	createCollection: {
		id: 'create.collection.create-collection',
		defaultMessage: 'Create collection',
	},
	errorTitle: {
		id: 'create.collection.error-title',
		defaultMessage: 'An error occurred',
	},
})

const name = ref('')
const description = ref('')
const hasHitLimit = ref(false)

const modal = ref()

const props = defineProps({
	projectIds: {
		type: Array,
		default() {
			return []
		},
	},
})

async function create() {
	startLoading()
	try {
		const result = await useBaseFetch('collection', {
			method: 'POST',
			body: {
				name: name.value.trim(),
				description: description.value.trim() || undefined,
				projects: props.projectIds,
			},
			apiVersion: 3,
		})

		await initUserCollections()

		modal.value.hide()
		await router.push(`/collection/${result.id}`)
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorTitle),
			text: err?.data?.description || err?.message || err,
			type: 'error',
		})
	}
	stopLoading()
}
function show(event) {
	name.value = ''
	description.value = ''
	modal.value.show(event)
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
