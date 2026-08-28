<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import {
	Accordion,
	commonMessages,
	defineMessages,
	InlineEditableText,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { nextTick, ref, watch } from 'vue'

const props = withDefaults(
	defineProps<{
		title: string
		count: number
		collapsed?: boolean
		forceOpen?: boolean
		hideHeader?: boolean
		editable?: boolean
		startEditing?: boolean
		maxTitleLength?: number
		validateTitle?: (value: string) => boolean
		onTitleChange?: (value: string) => boolean | void | Promise<boolean | void>
	}>(),
	{
		collapsed: false,
		forceOpen: false,
		hideHeader: false,
		editable: false,
		startEditing: false,
	},
)

const emit = defineEmits<{
	(e: 'update:collapsed', collapsed: boolean): void
}>()

const { formatMessage } = useVIntl()
const accordion = ref<InstanceType<typeof Accordion>>()
const titleInput = ref<InstanceType<typeof InlineEditableText>>()
const titleModel = ref(props.title)
const messages = defineMessages({
	collapse: { id: 'app.screenshots.group.collapse', defaultMessage: 'Collapse group' },
	expand: { id: 'app.screenshots.group.expand', defaultMessage: 'Expand group' },
})

function toggle() {
	if (accordion.value?.isOpen) {
		accordion.value.close()
	} else {
		accordion.value?.open()
	}
}

async function startTitleEditing() {
	if (!props.editable) return
	await titleInput.value?.startEditing()
}

watch(
	() => props.title,
	(title) => {
		titleModel.value = title
	},
)

watch(
	() => props.startEditing,
	async (shouldStartEditing) => {
		if (!shouldStartEditing) return
		await nextTick()
		await startTitleEditing()
	},
	{ immediate: true, flush: 'post' },
)
</script>

<template>
	<section class="flex w-full flex-col">
		<div
			v-if="!hideHeader"
			class="group/header flex h-10 w-full items-center gap-2 border-0 border-b border-solid border-b-surface-5"
		>
			<div class="group/open-target flex min-w-0 cursor-pointer items-center gap-2" @click="toggle">
				<button
					type="button"
					class="flex shrink-0 cursor-pointer items-center border-0 bg-transparent p-0"
					:aria-expanded="accordion?.isOpen"
					:aria-label="formatMessage(accordion?.isOpen ? messages.collapse : messages.expand)"
					@click.stop="toggle"
				>
					<DropdownIcon
						class="size-5 shrink-0 text-secondary transition-all duration-300 group-hover/open-target:text-primary"
						:class="{ 'rotate-180': accordion?.isOpen }"
					/>
				</button>
				<InlineEditableText
					v-if="editable"
					ref="titleInput"
					v-model="titleModel"
					activation-mode="manual"
					class="!h-10 select-none text-base font-semibold text-primary group-hover/open-target:text-contrast"
					:edit-label="formatMessage(commonMessages.renameButton)"
					max-width="24rem"
					icon-text-class="select-none"
					:max-length="maxTitleLength"
					:validate="validateTitle"
					:on-change="onTitleChange"
				/>
				<span
					v-else
					class="select-none truncate text-base font-semibold text-primary group-hover/open-target:text-contrast"
				>
					{{ title }}
				</span>
				<TagItem v-if="count" class="shrink-0 border-surface-3 bg-surface-2">
					{{ count }}
				</TagItem>
			</div>
			<div class="min-w-0 flex-1" />
			<slot name="actions" :start-editing="startTitleEditing" />
		</div>
		<Accordion
			ref="accordion"
			:open-by-default="hideHeader || !props.collapsed"
			:force-open="forceOpen"
			overflow-visible
			class="w-full"
			@on-open="emit('update:collapsed', false)"
			@on-close="emit('update:collapsed', true)"
		>
			<div class="mt-2.5">
				<slot />
			</div>
		</Accordion>
	</section>
</template>
