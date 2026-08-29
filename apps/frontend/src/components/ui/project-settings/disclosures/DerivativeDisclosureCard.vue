<script setup lang="ts">
import { GitForkIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	Input,
	SettingsFormGroup,
	Textarea,
	useVIntl,
} from '@modrinth/ui'
import { watch } from 'vue'

import DisclosureToggleCard from './DisclosureToggleCard.vue'
import type {
	DerivativeDisclosure,
	DerivativeSource,
	DisclosureCardMetaProps,
	DisclosureLockStatus,
} from './types'

const model = defineModel<DerivativeDisclosure>({ required: true })

const props = defineProps<DisclosureCardMetaProps>()

const emit = defineEmits<{
	setLockStatus: [status: DisclosureLockStatus]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.derivative.title',
		defaultMessage: 'Contains derivative content',
	},
	description: {
		id: 'project.settings.disclosures.derivative.description',
		defaultMessage:
			'You must enable this if your project is derivative of another project, such as being a fork or containing a substantial amount of someone else’s work.',
	},
	nameLabel: {
		id: 'project.settings.disclosures.derivative.name-label',
		defaultMessage: 'Name of original work',
	},
	namePlaceholder: {
		id: 'project.settings.disclosures.derivative.name-placeholder',
		defaultMessage: 'Example project',
	},
	linkLabel: {
		id: 'project.settings.disclosures.derivative.link-label',
		defaultMessage: 'Link to original work',
	},
	linkPlaceholder: {
		id: 'project.settings.disclosures.derivative.link-placeholder',
		defaultMessage: 'https://example.com',
	},
	noteLabel: {
		id: 'project.settings.disclosures.derivative.note-label',
		defaultMessage: 'Explain how your project is based on the original work',
	},
})

function emptyDerivativeSource(): DerivativeSource {
	return { label: '' }
}

function updateSource(index: number, patch: Partial<DerivativeSource>) {
	model.value.sources = model.value.sources.map((source, i) =>
		i === index ? { ...source, ...patch } : source,
	)
}

watch(
	() => model.value.enabled,
	(enabled) => {
		if (enabled && model.value.sources.length === 0) {
			model.value.sources = [emptyDerivativeSource()]
		}
	},
)

function addSource() {
	model.value.sources = [...model.value.sources, emptyDerivativeSource()]
}

function removeSource(index: number) {
	if (model.value.sources.length <= 1) return
	model.value.sources = model.value.sources.filter((_, i) => i !== index)
}

function setOptionalField(
	index: number,
	field: 'link' | 'note',
	value: string | number | undefined,
) {
	updateSource(index, {
		[field]: typeof value === 'string' && value ? value : null,
	})
}
</script>

<template>
	<DisclosureToggleCard
		v-bind="props"
		v-model="model.enabled"
		:icon="GitForkIcon"
		:title="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<template #expanded>
			<div
				v-for="(source, index) in model.sources"
				:key="index"
				class="relative flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
			>
				<div class="absolute right-3 top-3">
					<Button
						v-if="model.sources.length > 1"
						type="quiet"
						color="red"
						:disabled="disabled"
						@click="removeSource(index)"
					>
						<TrashIcon />
						{{ formatMessage(commonMessages.removeButton) }}
					</Button>
				</div>
				<SettingsFormGroup
					:title="formatMessage(messages.nameLabel)"
					:title-for="`derivative-name-${index}`"
					class="max-w-[40rem]"
				>
					<Input
						:id="`derivative-name-${index}`"
						:model-value="source.label"
						:disabled="disabled"
						:placeholder="formatMessage(messages.namePlaceholder)"
						@update:model-value="
							(value) => updateSource(index, { label: typeof value === 'string' ? value : '' })
						"
					/>
				</SettingsFormGroup>
				<SettingsFormGroup
					:title="formatMessage(messages.linkLabel)"
					:title-for="`derivative-link-${index}`"
					class="max-w-[40rem]"
				>
					<Input
						:id="`derivative-link-${index}`"
						:model-value="source.link ?? undefined"
						type="url"
						:disabled="disabled"
						:placeholder="formatMessage(messages.linkPlaceholder)"
						@update:model-value="(value) => setOptionalField(index, 'link', value)"
					/>
				</SettingsFormGroup>
				<SettingsFormGroup
					:title="formatMessage(messages.noteLabel)"
					:title-for="`derivative-note-${index}`"
					class="max-w-[40rem]"
				>
					<Textarea
						:id="`derivative-note-${index}`"
						:model-value="source.note ?? undefined"
						:rows="3"
						:disabled="disabled"
						@update:model-value="(value) => setOptionalField(index, 'note', value)"
					/>
				</SettingsFormGroup>
			</div>
			<Button class="w-fit" :disabled="disabled" @click="addSource">
				<PlusIcon />
				{{ formatMessage(commonMessages.addAnotherButton) }}
			</Button>
		</template>
	</DisclosureToggleCard>
</template>
