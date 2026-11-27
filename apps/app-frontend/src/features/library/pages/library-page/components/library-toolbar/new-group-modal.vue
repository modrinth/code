<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.createGroup)"
		no-padding
		scrollable
		actions-divider
		max-width="560px"
		width="560px"
	>
		<div class="flex flex-col gap-2.5 p-6">
			<label for="new-group-name" class="font-semibold text-contrast">
				{{ formatMessage(messages.groupName) }}
			</label>
			<Input
				id="new-group-name"
				ref="groupNameInput"
				v-model="name"
				:placeholder="formatMessage(messages.groupNamePlaceholder)"
				:maxlength="MAX_INSTANCE_GROUP_NAME_LENGTH"
				@click="groupNameInput?.select()"
			/>
		</div>

		<div class="h-px bg-surface-4" />

		<div class="flex h-[400px] flex-col gap-3 overflow-y-auto bg-surface-2 py-4">
			<div class="px-6">
				<Input
					v-model="search"
					type="search"
					:icon="SearchIcon"
					:aria-label="formatMessage(messages.searchInstance)"
					:placeholder="formatMessage(messages.searchInstance)"
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
						<div class="flex min-w-0 items-center gap-2">
							<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
							<TagItem v-if="instance.group_ids[0]" class="shrink-0">
								{{
									groupNamesById.get(instance.group_ids[0]) ?? formatMessage(messages.unknownGroup)
								}}
							</TagItem>
						</div>
					</div>
					<Button
						:type="selectedIds.has(instance.id) ? 'outlined' : 'base'"
						:aria-pressed="selectedIds.has(instance.id)"
						:aria-label="formatMessage(messages.addWithName, { name: instance.name })"
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
					{{ formatMessage(messages.cancel) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="normalizedName.length === 0"
					:loading="creatingGroup"
					@click="handleCreateGroup"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.createGroup) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, PlusIcon, SearchIcon, XIcon } from '@modrinth/assets'
import { Avatar, Button, defineMessages, Input, NewModal, TagItem, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { getInstanceIconUrl } from '@/helpers/instance'
import { MAX_INSTANCE_GROUP_NAME_LENGTH } from '@/platform/app-lib/instance-groups/types'

import {
	injectLibraryGroups,
	injectLibrarySelection,
	injectLibraryView,
} from '../../library-context'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	createGroup: { id: 'app.library.group.create', defaultMessage: 'Create group' },
	groupName: { id: 'app.library.group.name', defaultMessage: 'Group name' },
	groupNamePlaceholder: {
		id: 'app.library.group.name-placeholder',
		defaultMessage: 'Enter group name',
	},
	searchInstance: {
		id: 'app.library.group.search-instance',
		defaultMessage: 'Search instance',
	},
	noInstancesFound: {
		id: 'app.library.group.no-instances-found',
		defaultMessage: 'No instances found',
	},
	unknownGroup: { id: 'app.library.group.unknown', defaultMessage: 'Unknown group' },
	add: { id: 'app.library.group.instance.add', defaultMessage: 'Add' },
	added: { id: 'app.library.group.instance.added', defaultMessage: 'Added' },
	addWithName: { id: 'app.library.group.instance.add-with-name', defaultMessage: 'Add {name}' },
	cancel: { id: 'app.library.group.create.cancel', defaultMessage: 'Cancel' },
})

const { instances, groups } = injectLibraryView()
const { creatingGroup, createGroup, getDefaultGroupName } = injectLibraryGroups()
const { clearLibraryInstanceSelection } = injectLibrarySelection()

const modal = ref<InstanceType<typeof NewModal>>()
const groupNameInput = ref<InstanceType<typeof Input>>()
const name = ref('')
const search = ref('')
const selectedIds = ref(new Set<string>())

const normalizedName = computed(() =>
	name.value.trim().substring(0, MAX_INSTANCE_GROUP_NAME_LENGTH),
)
const groupNamesById = computed(() => new Map(groups.value.map((group) => [group.id, group.name])))
const visibleInstances = computed(() => {
	const query = search.value.trim().toLowerCase()
	return instances.value
		.filter((instance) => !query || instance.name.toLowerCase().includes(query))
		.sort((a, b) => {
			const groupedDifference = Number(a.group_ids.length > 0) - Number(b.group_ids.length > 0)
			return groupedDifference || a.name.localeCompare(b.name, undefined, { numeric: true })
		})
})

function toggleInstance(instanceId: string) {
	const nextIds = new Set(selectedIds.value)
	if (!nextIds.delete(instanceId)) nextIds.add(instanceId)
	selectedIds.value = nextIds
}

function show(instanceIds: Iterable<string> = [], event?: MouseEvent) {
	name.value = getDefaultGroupName()
	search.value = ''
	selectedIds.value = new Set(instanceIds)
	modal.value?.show(event)
}

async function handleCreateGroup() {
	if (normalizedName.value.length === 0 || creatingGroup.value) return
	const result = await createGroup(normalizedName.value, selectedIds.value)
	if (!result) return
	if (result.membershipsAdded) clearLibraryInstanceSelection()
	modal.value?.hide()
}

defineExpose({ show })
</script>
