<script setup lang="ts">
import { LinkIcon, TrashIcon, XIcon } from '@modrinth/assets'
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

type Choice = 'here' | 'all' | DesyncServerMode | null
const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const mode = ref<'change' | 'delete' | 'desync'>('change')
const action = ref<SyncedPackAction>('disable')
const items = ref<ContentItem[]>([])
const allowInstanceOverride = ref(false)
let resolveChoice: ((choice: Choice) => void) | undefined

const messages = defineMessages({
	removeResourcePackTitle: {
		id: 'app.synced-content.delete.resource-pack-title',
		defaultMessage: '{count, plural, one {Remove resource pack?} other {Remove resource packs?}}',
	},
	resourcePackHeader: {
		id: 'app.synced-content.delete.resource-pack-header',
		defaultMessage:
			'{count, plural, one {This resource pack is synced} other {These resource packs are synced}}',
	},
	resourcePackDescription: {
		id: 'app.synced-content.delete.resource-pack-description',
		defaultMessage:
			'You can remove {count, plural, one {it} other {them}} from just this instance or from all synced instances. Removing {count, plural, one {it} other {them}} from only this instance will enable overrides, and this instance will no longer receive synced resource pack changes.',
	},
	removeTitle: { id: 'app.synced-content.delete.title', defaultMessage: 'Remove content?' },
	removeDescription: {
		id: 'app.synced-content.delete.override-description',
		defaultMessage:
			'You can remove it from just this instance or from all synced instances. Removing it from only this instance will enable overrides, and this instance will no longer receive synced changes for these content types.',
	},
	removeHere: { id: 'app.synced-content.delete.remove-here', defaultMessage: 'Remove here' },
	removeEverywhere: {
		id: 'app.synced-content.delete.remove-everywhere',
		defaultMessage: 'Remove everywhere',
	},
	title: { id: 'app.synced-content.warning.title', defaultMessage: 'This content is synced' },
	enableTitle: { id: 'app.synced-content.change.enable-title', defaultMessage: 'Enable content?' },
	disableTitle: {
		id: 'app.synced-content.change.disable-title',
		defaultMessage: 'Disable content?',
	},
	enable: {
		id: 'app.synced-content.change.enable-description',
		defaultMessage:
			'Enable it across all synced instances, or only in this instance. Enabling it only here will turn on overrides for this instance.',
	},
	disable: {
		id: 'app.synced-content.change.disable-description',
		defaultMessage:
			'Disable it across all synced instances, or only in this instance. Disabling it only here will turn on overrides for this instance.',
	},
	enableEverywhereDescription: {
		id: 'app.synced-content.change.enable-everywhere-description',
		defaultMessage: 'Enable this content across all synced instances.',
	},
	disableEverywhereDescription: {
		id: 'app.synced-content.change.disable-everywhere-description',
		defaultMessage: 'Disable this content across all synced instances.',
	},
	enableHere: { id: 'app.synced-content.change.enable-here', defaultMessage: 'Enable here' },
	disableHere: { id: 'app.synced-content.change.disable-here', defaultMessage: 'Disable here' },
	enableEverywhere: {
		id: 'app.synced-content.change.enable-everywhere',
		defaultMessage: 'Enable everywhere',
	},
	disableEverywhere: {
		id: 'app.synced-content.change.disable-everywhere',
		defaultMessage: 'Disable everywhere',
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

const syncedItems = computed(() => items.value.filter((item) => item.synced_pack))
const removingResourcePacks = computed(
	() =>
		mode.value === 'delete' &&
		syncedItems.value.length > 0 &&
		syncedItems.value.every((item) => item.project_type === 'resourcepack'),
)

const title = computed(
	() =>
		({
			change: action.value === 'enable' ? messages.enableTitle : messages.disableTitle,
			delete: removingResourcePacks.value ? messages.removeResourcePackTitle : messages.removeTitle,
			desync: messages.desyncTitle,
		})[mode.value],
)
const description = computed(() => {
	if (mode.value === 'desync') return messages.desyncDescription
	if (mode.value === 'delete') {
		if (!allowInstanceOverride.value) return messages.deleteDescription
		return removingResourcePacks.value
			? messages.resourcePackDescription
			: messages.removeDescription
	}
	if (!allowInstanceOverride.value) {
		return action.value === 'enable'
			? messages.enableEverywhereDescription
			: messages.disableEverywhereDescription
	}
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
	settle(null)
	if (!modal.value) return Promise.resolve(null)
	return new Promise<Choice>((resolve) => {
		resolveChoice = resolve
		modal.value?.show()
	})
}

async function confirmChange(value: SyncedPackAction, content: ContentItem[], canOverride = false) {
	allowInstanceOverride.value = canOverride
	mode.value = 'change'
	action.value = value
	items.value = content.filter((item) => item.synced_pack)
	if (items.value.length === 0) return 'all'
	const choice = await show()
	return choice === 'all' || (canOverride && choice === 'here') ? choice : null
}

async function confirmDelete(content: ContentItem[], canOverride = false) {
	allowInstanceOverride.value = canOverride
	mode.value = 'delete'
	items.value = content
	const choice = await show()
	return choice === 'all' || (canOverride && choice === 'here') ? choice : null
}

async function confirmDesync(item: ContentItem) {
	mode.value = 'desync'
	items.value = [item]
	const choice = await show()
	return choice === 'keep_in_other_instances' || choice === 'remove_from_other_instances'
		? choice
		: null
}

onBeforeUnmount(() => settle(null))
defineExpose({ confirmChange, confirmDelete, confirmDesync })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(title, { count: syncedItems.length })"
		fade="warning"
		max-width="560px"
		@hide="settle(null)"
	>
		<div class="flex flex-col gap-6">
			<Admonition
				type="warning"
				:header="
					formatMessage(removingResourcePacks ? messages.resourcePackHeader : messages.title, {
						count: syncedItems.length,
					})
				"
			>
				{{ formatMessage(description, { count: syncedItems.length }) }}
			</Admonition>
			<div v-if="mode === 'desync'" class="flex max-h-[212px] flex-col gap-2 overflow-y-auto">
				<div
					v-for="item in items"
					:key="item.id"
					class="rounded-xl border border-solid border-surface-4 bg-surface-2 p-4"
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
				<Button type="outlined" @click="finish(null)">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<template v-if="mode === 'desync'">
					<Button @click="finish('keep_in_other_instances')">
						<LinkIcon aria-hidden="true" />
						{{ formatMessage(messages.keep) }}
					</Button>
					<Button type="colored" color="orange" @click="finish('remove_from_other_instances')">
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(messages.remove) }}
					</Button>
				</template>
				<template v-else-if="mode === 'change'">
					<Button
						:type="allowInstanceOverride ? 'outlined' : 'colored'"
						color="orange"
						@click="finish('all')"
					>
						{{
							formatMessage(
								action === 'enable' ? messages.enableEverywhere : messages.disableEverywhere,
							)
						}}
					</Button>
					<Button
						v-if="allowInstanceOverride"
						type="colored"
						color="orange"
						@click="finish('here')"
					>
						{{ formatMessage(action === 'enable' ? messages.enableHere : messages.disableHere) }}
					</Button>
				</template>
				<template v-else>
					<Button
						:type="allowInstanceOverride ? 'outlined' : 'colored'"
						color="orange"
						@click="finish('all')"
					>
						{{ formatMessage(messages.removeEverywhere) }}
					</Button>
					<Button
						v-if="allowInstanceOverride"
						type="colored"
						color="orange"
						@click="finish('here')"
					>
						{{ formatMessage(messages.removeHere) }}
					</Button>
				</template>
			</div>
		</template>
	</NewModal>
</template>
