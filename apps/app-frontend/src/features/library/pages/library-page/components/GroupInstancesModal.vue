<template>
	<NewModal
		ref="modal"
		:header="title"
		no-padding
		scrollable
		actions-divider
		max-width="560px"
		width="560px"
	>
		<div class="flex h-[400px] flex-col gap-3 overflow-y-auto bg-surface-2 py-4">
			<div class="px-6">
				<Input
					v-model="search"
					type="search"
					:icon="SearchIcon"
					:aria-label="formatMessage(messages.searchPlaceholder)"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					wrapper-class="w-full"
				/>
			</div>

			<div
				v-if="visibleInstances.length === 0"
				class="flex items-center justify-center py-12 text-secondary"
			>
				{{ formatMessage(messages.noInstancesFound) }}
			</div>
			<div v-else class="flex flex-col gap-1">
				<div
					v-for="instance in visibleInstances"
					:key="instance.id"
					class="flex items-center justify-between gap-4 px-6 py-1.5 hover:bg-surface-3"
					:class="{ 'opacity-60': selectedIds.has(instance.id) }"
				>
					<div class="flex min-w-0 items-center gap-2.5">
						<Avatar
							:src="getInstanceIconUrl(instance.icon_path)"
							:tint-by="instance.id"
							:alt="instance.name"
							size="2rem"
							rounded="md"
							pad-transparent-corners
						/>
						<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
					</div>
					<Button
						:type="selectedIds.has(instance.id) ? 'outlined' : 'base'"
						:aria-pressed="selectedIds.has(instance.id)"
						:aria-label="formatMessage(messages.addWithName, { name: instance.name })"
						:disabled="isUngrouped && selectedIds.has(instance.id)"
						@click="toggleInstance(instance.id)"
					>
						<CheckIcon v-if="selectedIds.has(instance.id)" aria-hidden="true" />
						{{ formatMessage(selectedIds.has(instance.id) ? messages.added : messages.add) }}
					</Button>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :loading="saving" @click="save">
					<CheckIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.saveChangesButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, SearchIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { getInstanceIconUrl } from '@/helpers/instance'

import { injectLibraryGroups, injectLibrarySelection, injectLibraryView } from '../library-context'
import {
	getLibraryInstanceSelectionKey,
	type LibraryInstanceSelection,
	UNGROUPED_GROUP_ID,
} from '../types'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: {
		id: 'app.library.group.instances-modal.title',
		defaultMessage: 'Add instances to "{groupName}"',
	},
	titleUngrouped: {
		id: 'app.library.group.instances-modal.title-ungrouped',
		defaultMessage: 'Add instances to "Ungrouped"',
	},
	searchPlaceholder: {
		id: 'app.library.group.instances-modal.search-placeholder',
		defaultMessage: 'Search instance',
	},
	noInstancesFound: {
		id: 'app.library.group.instances-modal.no-instances-found',
		defaultMessage: 'No instances found',
	},
	add: {
		id: 'app.library.group.instances-modal.add',
		defaultMessage: 'Add',
	},
	added: {
		id: 'app.library.group.instances-modal.added',
		defaultMessage: 'Added',
	},
	addWithName: {
		id: 'app.library.group.instances-modal.add-with-name',
		defaultMessage: 'Add {name}',
	},
})

const { instances, groups } = injectLibraryView()
const { setMemberships } = injectLibraryGroups()
const { selectedLibraryInstances, setSelectedLibraryInstances } = injectLibrarySelection()

const modal = ref<InstanceType<typeof NewModal>>()
const groupId = ref<string | null>(null)
const search = ref('')
const selectedIds = ref(new Set<string>())
const saving = computed(() => setMemberships.isPending.value)

const isUngrouped = computed(() => groupId.value === UNGROUPED_GROUP_ID)
const title = computed(() =>
	isUngrouped.value
		? formatMessage(messages.titleUngrouped)
		: formatMessage(messages.title, {
				groupName: groups.value.find((group) => group.id === groupId.value)?.name ?? '',
			}),
)
const visibleInstances = computed(() => {
	const query = search.value.trim().toLowerCase()
	return instances.value
		.filter((instance) => !query || instance.name.toLowerCase().includes(query))
		.sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true }))
})

function show(targetGroupId: string, event?: MouseEvent) {
	if (
		targetGroupId !== UNGROUPED_GROUP_ID &&
		!groups.value.some((group) => group.id === targetGroupId)
	) {
		return
	}

	groupId.value = targetGroupId
	search.value = ''
	selectedIds.value = new Set(
		instances.value
			.filter((instance) =>
				targetGroupId === UNGROUPED_GROUP_ID
					? instance.group_ids.length === 0
					: instance.group_ids.includes(targetGroupId),
			)
			.map((instance) => instance.id),
	)
	modal.value?.show(event)
}

function toggleInstance(instanceId: string) {
	const nextIds = new Set(selectedIds.value)
	if (nextIds.has(instanceId)) {
		if (isUngrouped.value) return
		nextIds.delete(instanceId)
	} else {
		nextIds.add(instanceId)
	}
	selectedIds.value = nextIds
}

function getNextSelection(
	operations: { instanceId: string; included: boolean; nextGroupIds: string[] }[],
	targetGroupId: string,
) {
	const nextSelection = new Map(selectedLibraryInstances.value)
	for (const { instanceId, included, nextGroupIds } of operations) {
		const instanceSelections = [...nextSelection.values()].filter(
			(selection) => selection.instanceId === instanceId,
		)
		if (included && instanceSelections.length > 0) {
			for (const selection of instanceSelections) {
				nextSelection.delete(getLibraryInstanceSelectionKey(selection))
			}
			const destination = { instanceId, groupId: targetGroupId }
			nextSelection.set(getLibraryInstanceSelectionKey(destination), destination)
			continue
		}

		const validGroupIds = new Set(nextGroupIds.length > 0 ? nextGroupIds : [UNGROUPED_GROUP_ID])
		const invalidSelections = instanceSelections.filter(
			(selection) => !validGroupIds.has(selection.groupId),
		)
		for (const selection of invalidSelections) {
			nextSelection.delete(getLibraryInstanceSelectionKey(selection))
		}

		const hasValidSelection = [...nextSelection.values()].some(
			(selection) => selection.instanceId === instanceId,
		)
		if (invalidSelections.length > 0 && !hasValidSelection) {
			const replacement: LibraryInstanceSelection = {
				instanceId,
				groupId: nextGroupIds[0] ?? UNGROUPED_GROUP_ID,
			}
			nextSelection.set(getLibraryInstanceSelectionKey(replacement), replacement)
		}
	}
	return nextSelection.values()
}

async function save() {
	const targetGroupId = groupId.value
	if (!targetGroupId || saving.value) return

	const operations = instances.value
		.filter((instance) => {
			const isSelected = selectedIds.value.has(instance.id)
			return isUngrouped.value
				? isSelected && instance.group_ids.length > 0
				: instance.group_ids.includes(targetGroupId) !== isSelected
		})
		.map((instance) => {
			const included = selectedIds.value.has(instance.id)
			const nextGroupIds = isUngrouped.value
				? []
				: included
					? [...instance.group_ids, targetGroupId]
					: instance.group_ids.filter((instanceGroupId) => instanceGroupId !== targetGroupId)
			return { instanceId: instance.id, included, nextGroupIds }
		})

	try {
		await setMemberships.mutateAsync(
			operations.map(({ instanceId, nextGroupIds }) => ({
				instance_id: instanceId,
				group_ids: nextGroupIds,
			})),
		)
	} catch {
		return
	}

	setSelectedLibraryInstances(getNextSelection(operations, targetGroupId))
	modal.value?.hide()
}

defineExpose({ show })
</script>
