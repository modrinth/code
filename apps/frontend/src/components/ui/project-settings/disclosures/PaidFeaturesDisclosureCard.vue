<script setup lang="ts">
import { CircleDollarSignIcon, ListPlusIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	Input,
	SettingsFormGroup,
	useVIntl,
} from '@modrinth/ui'
import { watch } from 'vue'

import DisclosureToggleCard from './DisclosureToggleCard.vue'
import type { DisclosureCardMetaProps, DisclosureLockStatus, PaidFeaturesDisclosure } from './types'

const model = defineModel<PaidFeaturesDisclosure>({ required: true })

const props = defineProps<DisclosureCardMetaProps>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.paid-features.title',
		defaultMessage: 'Contains paid features',
	},
	description: {
		id: 'project.settings.disclosures.paid-features.description',
		defaultMessage:
			'You must enable this if your project contains features that can be obtained by spending real-world money.',
	},
	featuresDescription: {
		id: 'project.settings.disclosures.paid-features.features-description',
		defaultMessage: 'List the types of paid features',
	},
	featurePlaceholder: {
		id: 'project.settings.disclosures.paid-features.feature-placeholder',
		defaultMessage: 'e.g. Cosmetics available as Patreon reward',
	},
})

watch(
	() => model.value.enabled,
	(enabled) => {
		if (enabled && model.value.features.length === 0) {
			model.value.features = ['']
		}
	},
)

function addFeature() {
	model.value.features = [...model.value.features, '']
}

function removeFeature(index: number) {
	if (model.value.features.length <= 1) return
	model.value.features = model.value.features.filter((_, i) => i !== index)
}
</script>

<template>
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="CircleDollarSignIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<template #expanded>
			<SettingsFormGroup :title="formatMessage(messages.featuresDescription)">
				<div v-for="(_, index) in model.features" :key="index" class="flex items-center gap-2">
					<Input
						v-model="model.features[index]"
						class="min-w-0 flex-1"
						:disabled="disabled"
						:placeholder="formatMessage(messages.featurePlaceholder)"
					/>
					<IconButton
						v-if="model.features.length > 1"
						:label="formatMessage(commonMessages.removeButton)"
						:disabled="disabled"
						@click="removeFeature(index)"
					>
						<XIcon />
					</IconButton>
				</div>
			</SettingsFormGroup>
			<Button class="w-fit" :disabled="disabled" @click="addFeature">
				<ListPlusIcon />
				{{ formatMessage(commonMessages.addAnotherButton) }}
			</Button>
		</template>
	</DisclosureToggleCard>
</template>
