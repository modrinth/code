<script setup lang="ts">
import { LinkIcon, RightArrowIcon, TrashIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	commonMessages,
	ContentCardItem,
	type ContentItem,
	defineMessages,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, ref } from 'vue'

import type { SyncedPackAction } from '@/helpers/synced-packs'
import type { DesyncServerMode } from '@/helpers/worlds'

type Choice = 'confirm' | 'all' | DesyncServerMode | null
const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const mode = ref<'change' | 'delete' | 'desync'>('change')
const action = ref<SyncedPackAction>('disable')
const items = ref<ContentItem[]>([])
let resolveChoice: ((choice: Choice) => void) | undefined

const messages = defineMessages({
	warningTitle: { id: 'app.synced-content.warning.modal-title', defaultMessage: 'Sync warning' },
	title: { id: 'app.synced-content.warning.title', defaultMessage: 'This content is synced' },
	enable: {
		id: 'app.synced-content.warning.enable',
		defaultMessage:
			'Enabling this content will also enable it in the other instances where it is synced. To change it only here, desync it first.',
	},
	disable: {
		id: 'app.synced-content.warning.disable',
		defaultMessage:
			'Disabling this content will also disable it in the other instances where it is synced. To change it only here, desync it first.',
	},
	desyncTitle: { id: 'app.synced-content.desync.title', defaultMessage: 'Desync content' },
	desyncDescription: {
		id: 'app.synced-content.desync.description',
		defaultMessage:
			'This instance will keep its own copy. Do you want to keep this content synced in other instances, or remove it from them?',
	},
	keep: { id: 'app.synced-content.desync.keep', defaultMessage: 'Keep' },
	remove: { id: 'app.synced-content.desync.remove', defaultMessage: 'Remove' },
	deleteDescription: {
		id: 'app.synced-content.delete.description',
		defaultMessage: 'Deleting this content will delete it across all your instances',
	},
	mixedDeletion: {
		id: 'app.synced-content.delete.mixed',
		defaultMessage: 'Selected content that is not synced will only be deleted from this instance.',
	},
})

const title = computed(
	() =>
		({
			change: messages.warningTitle,
			delete: messages.warningTitle,
			desync: messages.desyncTitle,
		})[mode.value],
)
const description = computed(() => {
	if (mode.value === 'desync') return messages.desyncDescription
	if (mode.value === 'delete') return messages.deleteDescription
	return action.value === 'enable' ? messages.enable : messages.disable
})

function settle(choice: Choice) {
	const resolve = resolveChoice
	resolveChoice = undefined
	resolve?.(choice)
}

function finish(choice: Choice) {
	settle(choice)
	modal.value?.hide()
}

function show() {
	resolveChoice?.(null)
	return new Promise<Choice>((resolve) => {
		resolveChoice = resolve
		modal.value?.show()
	})
}

async function confirmChange(value: SyncedPackAction, content: ContentItem[]) {
	mode.value = 'change'
	action.value = value
	items.value = content.filter((item) => item.synced_pack)
	if (items.value.length === 0) return true
	return (await show()) === 'confirm'
}

async function confirmDelete(content: ContentItem[]) {
	mode.value = 'delete'
	items.value = content
	const choice = await show()
	return choice === 'all' ? choice : null
}

async function confirmDesync(item: ContentItem) {
	mode.value = 'desync'
	items.value = [item]
	const choice = await show()
	return choice === 'keep_in_other_instances' || choice === 'remove_from_other_instances'
		? choice
		: null
}

onBeforeUnmount(() => resolveChoice?.(null))
defineExpose({ confirmChange, confirmDelete, confirmDesync })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(title)"
		:fade="mode === 'delete' ? 'danger' : 'warning'"
		max-width="560px"
		@hide="settle(null)"
	>
		<div class="flex flex-col gap-6">
			<Admonition
				:type="mode === 'delete' ? 'critical' : 'warning'"
				:header="formatMessage(messages.title)"
			>
				{{ formatMessage(description) }}
			</Admonition>
			<div v-if="mode === 'desync'" class="flex max-h-[212px] flex-col gap-2 overflow-y-auto">
				<div
					v-for="item in items"
					:key="item.id"
					class="rounded-xl border border-solid border-surface-5 p-4 !bg-surface-2"
				>
					<ContentCardItem
						:project="
							item.project ?? {
								id: item.id,
								slug: null,
								title: item.embedded_metadata?.name ?? item.file_name,
								icon_url: item.embedded_metadata?.icon_url ?? null,
							}
						"
						:project-link="
							!item.external && item.project?.id ? `/project/${item.project.id}` : undefined
						"
						:version="
							item.version ?? {
								id: item.id,
								version_number:
									item.embedded_metadata?.version ?? formatMessage(commonMessages.unknownLabel),
								file_name: item.file_name,
							}
						"
						:owner="item.owner"
						hide-actions
						inline
					/>
				</div>
			</div>
			<p
				v-if="mode === 'delete' && items.some((item) => !item.synced_pack)"
				class="m-0 text-secondary"
			>
				{{ formatMessage(messages.mixedDeletion) }}
			</p>
		</div>
		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<Button type="outlined" class="!border !border-surface-5" @click="finish(null)"
					><XIcon />{{ formatMessage(commonMessages.cancelButton) }}</Button
				>
				<template v-if="mode === 'desync'">
					<Button @click="finish('keep_in_other_instances')"
						><LinkIcon />{{ formatMessage(messages.keep) }}</Button
					>
					<Button type="colored" color="orange" @click="finish('remove_from_other_instances')"
						><TrashIcon />{{ formatMessage(messages.remove) }}</Button
					>
				</template>
				<Button
					v-else
					type="colored"
					:color="mode === 'delete' ? 'red' : 'orange'"
					@click="finish(mode === 'delete' ? 'all' : 'confirm')"
				>
					<TrashIcon v-if="mode === 'delete'" />
					<RightArrowIcon v-else />
					{{
						formatMessage(
							mode === 'delete' ? commonMessages.deleteLabel : commonMessages.continueButton,
						)
					}}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
