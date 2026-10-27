<script setup lang="ts">
import { ExternalIcon, GitForkIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	Input,
	SettingsFormGroup,
	useVIntl,
} from '@modrinth/ui'
import { computed, toRaw, useId, watch } from 'vue'

import DisclosureExplanation from './DisclosureExplanation.vue'
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
const sourceIdPrefix = useId()
const sourceIds = new WeakMap<DerivativeSource, string>()
let nextSourceId = 0
const sources = computed(() =>
	model.value.sources.map((source) => ({ source, id: getSourceId(source) })),
)

function getSourceId(source: DerivativeSource): string {
	const rawSource = toRaw(source)
	let id = sourceIds.get(rawSource)
	if (!id) {
		id = `${sourceIdPrefix}-${nextSourceId++}`
		sourceIds.set(rawSource, id)
	}
	return id
}

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
	openLink: {
		id: 'project.settings.disclosures.derivative.open-link',
		defaultMessage: 'Visit link',
	},
	linkPlaceholder: {
		id: 'project.settings.disclosures.derivative.link-placeholder',
		defaultMessage: 'https://example.com',
	},
	noteLabel: {
		id: 'project.settings.disclosures.derivative.note-label',
		defaultMessage: 'Explain how your project is based on the original work',
	},
	notePlaceholder: {
		id: 'project.settings.disclosures.derivative.note-placeholder',
		defaultMessage:
			'This is a fork that adds a few new biomes and ports it to newer Minecraft versions.',
	},
})

function emptyDerivativeSource(): DerivativeSource {
	return { label: '' }
}

function updateSource(index: number, patch: Partial<DerivativeSource>) {
	model.value.sources = model.value.sources.map((source, i) => {
		if (i !== index) return source
		const updated = { ...source, ...patch }
		sourceIds.set(updated, getSourceId(source))
		return updated
	})
}

watch(
	() => model.value.enabled,
	(enabled) => {
		if (enabled && model.value.sources.length === 0) {
			model.value.sources = [emptyDerivativeSource()]
		}
	},
	{ immediate: true },
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
		:title="props.title ?? formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
		info-link="https://support.modrinth.com/en/articles/16567675#h_6422cc821e"
		@set-lock-status="emit('setLockStatus', $event)"
	>
		<template #expanded>
			<div
				v-for="({ source, id }, index) in sources"
				:key="id"
				class="relative flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
				:class="{ 'mt-8': variant === 'review' && editing && index === 0 }"
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
					:title-for="`derivative-name-${id}`"
					class="max-w-[40rem]"
				>
					<Input
						:id="`derivative-name-${id}`"
						:model-value="source.label"
						:disabled="disabled"
						:placeholder="formatMessage(messages.namePlaceholder)"
						@update:model-value="
							(value) =>
								updateSource(index, {
									label: typeof value === 'string' ? value : '',
								})
						"
					/>
				</SettingsFormGroup>
				<SettingsFormGroup
					:title="formatMessage(messages.linkLabel)"
					:title-for="`derivative-link-${id}`"
					class="max-w-[40rem]"
				>
					<template #title-right>
						<ButtonLink
							v-if="source.link"
							:href="/^https?:\/\//i.test(source.link) ? source.link : `https://${source.link}`"
							target="_blank"
							type="quiet"
							size="sm"
							class="-my-1.5"
						>
							{{ formatMessage(messages.openLink) }}
							<ExternalIcon />
						</ButtonLink>
					</template>
					<Input
						:id="`derivative-link-${id}`"
						:model-value="source.link ?? undefined"
						type="url"
						:disabled="disabled"
						:placeholder="formatMessage(messages.linkPlaceholder)"
						@update:model-value="(value) => setOptionalField(index, 'link', value)"
					/>
				</SettingsFormGroup>
				<DisclosureExplanation
					:title="formatMessage(messages.noteLabel)"
					:title-for="`derivative-note-${id}`"
					:variant="variant"
					:model-value="source.note ?? undefined"
					:resize="showLockControls ? 'vertical' : 'none'"
					:disabled="disabled"
					:placeholder="formatMessage(messages.notePlaceholder)"
					@update:model-value="(value) => setOptionalField(index, 'note', value)"
				/>
			</div>
			<Button class="w-fit" :disabled="disabled" @click="addSource">
				<PlusIcon />
				{{ formatMessage(commonMessages.addAnotherButton) }}
			</Button>
		</template>
	</DisclosureToggleCard>
</template>
