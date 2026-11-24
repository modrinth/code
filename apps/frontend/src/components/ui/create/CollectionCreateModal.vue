<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		max-width="550px"
		:initial-focus="getNameInput"
	>
		<div class="flex w-full flex-col gap-6">
			<RestrictedActionAlert
				v-model="isRestricted"
				scope="COLLECTION_CREATE"
				@navigate="modal.hide()"
			/>
			<CreateLimitAlert v-model="hasHitLimit" type="collection" />
			<Field :label="formatMessage(messages.nameLabel)" :input-id="nameId">
				<Input
					:id="nameId"
					ref="nameInput"
					v-model="name"
					:maxlength="64"
					:placeholder="formatMessage(messages.namePlaceholder)"
					autocomplete="off"
					wrapper-class="w-full"
					aria-required="true"
					:disabled="hasHitLimit"
				/>
			</Field>
			<Field :label="formatMessage(messages.summaryLabel)" :input-id="summaryId" label-optional>
				<Textarea
					:id="summaryId"
					v-model="description"
					:maxlength="255"
					:placeholder="formatMessage(messages.summaryPlaceholder)"
					:disabled="hasHitLimit"
					:aria-describedby="summaryHelpId"
				/>
				<template #subtext>
					<span :id="summaryHelpId">{{ formatMessage(messages.summaryDescription) }}</span>
				</template>
			</Field>
			<div class="flex flex-wrap justify-end gap-2.5">
				<Button type="outlined" @click="modal.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					v-tooltip="validationTooltip"
					type="colored"
					color="brand"
					:disabled="disableCreate"
					@click="create"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.createCollection) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>
<script setup>
import { PlusIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	Field,
	injectNotificationManager,
	Input,
	NewModal,
	Textarea,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, useId } from 'vue'

import CreateLimitAlert from './CreateLimitAlert.vue'
import RestrictedActionAlert from './RestrictedActionAlert.vue'

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
	createCollection: {
		id: 'create.collection.create-collection',
		defaultMessage: 'Create collection',
	},
	invalidName: {
		id: 'create.collection.invalid-name',
		defaultMessage: 'Enter a collection name of 3 to 64 characters.',
	},
	invalidSummary: {
		id: 'create.collection.invalid-summary',
		defaultMessage: 'Enter a summary of 3 to 255 characters or leave it blank.',
	},
})

const name = ref('')
const description = ref('')
const hasHitLimit = ref(false)
const isRestricted = ref(false)
const isCreating = ref(false)

const modal = ref()
const nameInput = ref()
const nameId = useId()
const summaryId = useId()
const summaryHelpId = useId()

const validationTooltip = computed(() => {
	const nameLength = [...name.value.trim()].length
	const summaryLength = [...description.value.trim()].length
	if (nameLength < 3 || nameLength > 64) return formatMessage(messages.invalidName)
	if (summaryLength > 0 && (summaryLength < 3 || summaryLength > 255))
		return formatMessage(messages.invalidSummary)
	return ''
})
const disableCreate = computed(
	() => hasHitLimit.value || isRestricted.value || isCreating.value || !!validationTooltip.value,
)

const props = defineProps({
	projectIds: {
		type: Array,
		default() {
			return []
		},
	},
})

async function create() {
	if (disableCreate.value) return
	isCreating.value = true
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
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err?.data?.description || err?.message || err,
			type: 'error',
		})
	} finally {
		isCreating.value = false
		stopLoading()
	}
}
function show(event) {
	name.value = ''
	description.value = ''
	modal.value.show(event)
}

function getNameInput() {
	return nameInput.value?.$el.querySelector('input:enabled') ?? null
}

defineExpose({
	show,
})
</script>
