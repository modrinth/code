<script setup lang="ts">
import { ChevronDownIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	SettingsFormGroup,
	Textarea,
	useVIntl,
} from '@modrinth/ui'
import { nextTick, onScopeDispose, ref, useId, watch } from 'vue'

import EditButton from '~/components/project-review/project-info/edit/button.vue'
import { injectReviewShortcuts } from '~/components/project-review/shortcuts'

import { injectDisclosureDraft } from './draft-context'

const props = defineProps<{
	variant?: 'settings' | 'review'
	title: string
	titleFor?: string
	optional?: boolean
	disabled?: boolean
	placeholder?: string
	resize?: 'none' | 'vertical' | 'both'
}>()
const model = defineModel<string | undefined>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	empty: {
		id: 'project-review.disclosures.empty-explanation',
		defaultMessage: 'No explanation provided.',
	},
	done: {
		id: 'project-review.disclosures.done-editing',
		defaultMessage: 'Done',
	},
})
const expanded = ref(true)
const editing = ref(false)
const original = ref<string>()
const section = ref<HTMLElement>()
const textarea = ref<InstanceType<typeof Textarea>>()
const contentId = useId()
const draft = injectDisclosureDraft(null)
if (draft) {
	watch(draft, () => {
		editing.value = false
		original.value = undefined
	})
}
const shortcuts = injectReviewShortcuts(null)
if (shortcuts) {
	onScopeDispose(
		shortcuts.registerInteraction({
			element: () => (props.variant === 'review' ? section.value : undefined),
			collapse: () => {
				expanded.value = !expanded.value
			},
		}),
	)
}

watch(
	() => props.disabled,
	(disabled) => {
		if (disabled) editing.value = false
	},
)

async function startEditing() {
	if (props.disabled) return
	original.value = model.value
	expanded.value = true
	editing.value = true
	await nextTick()
	textarea.value?.focus()
}

function cancelEditing() {
	model.value = original.value
	editing.value = false
}
</script>

<template>
	<section
		v-if="variant === 'review'"
		ref="section"
		class="editable-review-section flex min-w-0 flex-col gap-2"
	>
		<div class="flex items-center justify-between gap-2">
			<button
				type="button"
				class="flex items-center gap-1 border-0 bg-transparent p-0 font-semibold text-contrast"
				:aria-expanded="expanded"
				:aria-controls="contentId"
				@click="expanded = !expanded"
			>
				{{ title }}
				<ChevronDownIcon
					class="size-4 transition-transform"
					:class="{ 'rotate-180': expanded }"
					aria-hidden="true"
				/>
			</button>
			<div v-if="editing" class="flex shrink-0 gap-1">
				<Button type="quiet" size="xs" :disabled="disabled" @click="cancelEditing">
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					size="xs"
					:disabled="disabled"
					@click="editing = false"
				>
					{{ formatMessage(messages.done) }}
				</Button>
			</div>
			<EditButton v-else :section="title" :disabled="disabled" @click="startEditing" />
		</div>
		<div
			:id="contentId"
			class="-m-1 grid transition-[grid-template-rows] duration-300 ease-in-out motion-reduce:transition-none"
			:class="expanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
			:inert="!expanded"
		>
			<div class="min-h-0 overflow-hidden">
				<div class="p-1">
					<Textarea
						v-if="editing"
						ref="textarea"
						v-model="model"
						:aria-label="title"
						:disabled="disabled"
						:placeholder="placeholder"
						:rows="3"
						resize="vertical"
						class="w-full"
					/>
					<p
						v-else
						class="m-0 whitespace-pre-wrap break-words"
						:class="{ 'text-secondary': !model }"
					>
						{{ model || formatMessage(messages.empty) }}
					</p>
				</div>
			</div>
		</div>
	</section>
	<SettingsFormGroup v-else :title="title" :title-for="titleFor" :optional="optional">
		<Textarea
			:id="titleFor"
			v-model="model"
			:rows="3"
			:resize="resize"
			class="max-w-[40rem]"
			:disabled="disabled"
			:placeholder="placeholder"
		/>
	</SettingsFormGroup>
</template>
