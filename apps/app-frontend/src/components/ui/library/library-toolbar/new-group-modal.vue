<template>
	<NewModal
		ref="modal"
		no-padding
		scrollable
		actions-divider
		max-width="560px"
		width="560px"
		:on-hide="closeNewGroupModal"
	>
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.createGroup) }}
			</span>
		</template>

		<div class="flex flex-col gap-2.5 p-6">
			<label for="new-group-name" class="font-semibold text-contrast">
				{{ formatMessage(messages.groupName) }}
			</label>
			<StyledInput
				id="new-group-name"
				ref="groupNameInput"
				v-model="newGroupName"
				:placeholder="formatMessage(messages.groupNamePlaceholder)"
				:maxlength="MAX_INSTANCE_GROUP_NAME_LENGTH"
				@click="groupNameInput?.select()"
			/>
		</div>

		<div class="h-px bg-divider" />

		<div class="flex h-[400px] flex-col gap-3 overflow-y-auto bg-surface-2 py-4">
			<div class="px-6">
				<StyledInput
					v-model="newGroupSearch"
					:icon="SearchIcon"
					:placeholder="formatMessage(messages.searchInstance)"
					class="w-full"
				/>
			</div>

			<div
				v-if="newGroupInstances.length === 0"
				class="flex items-center justify-center py-12 text-secondary"
			>
				{{ formatMessage(messages.noInstancesFound) }}
			</div>
			<div v-else class="flex flex-col gap-1">
				<div
					v-for="instance in newGroupInstances"
					:key="instance.id"
					class="flex items-center justify-between gap-4 px-6 py-1.5 hover:bg-surface-3"
					:class="{ 'opacity-60': selectedNewGroupInstanceIds.has(instance.id) }"
				>
					<div class="flex min-w-0 items-center gap-2.5">
						<Avatar
							:src="getInstanceIconUrl(instance.icon_path)"
							:tint-by="instance.id"
							:alt="instance.name"
							size="2rem"
							rounded="md"
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
						:type="selectedNewGroupInstanceIds.has(instance.id) ? 'outlined' : 'base'"
						@click="toggleNewGroupInstance(instance.id)"
					>
						<CheckIcon v-if="selectedNewGroupInstanceIds.has(instance.id)" />
						{{
							formatMessage(
								selectedNewGroupInstanceIds.has(instance.id) ? messages.added : messages.add,
							)
						}}
					</Button>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(messages.cancel) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!canCreateGroup" @click="handleCreateGroup">
					<SpinnerIcon v-if="creatingGroup" class="animate-spin" />
					<PlusIcon v-else />
					{{ formatMessage(messages.createGroup) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, PlusIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	NewModal,
	StyledInput,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'
import { getInstanceIconUrl } from '@/helpers/instance'
import { MAX_INSTANCE_GROUP_NAME_LENGTH } from '@/helpers/instance-groups'

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
	cancel: { id: 'app.library.group.create.cancel', defaultMessage: 'Cancel' },
})

const {
	isNewGroupModalOpen,
	libraryGroups,
	newGroupName,
	newGroupSearch,
	selectedNewGroupInstanceIds,
	creatingGroup,
	newGroupInstances,
	canCreateGroup,
	closeNewGroupModal,
	toggleNewGroupInstance,
	createGroup,
	clearLibraryInstanceSelection,
} = useLibrary()

const modal = ref<InstanceType<typeof NewModal>>()
const groupNameInput = ref<InstanceType<typeof StyledInput>>()
const groupNamesById = computed(
	() => new Map(libraryGroups.value.map((group) => [group.id, group.name])),
)

watch(isNewGroupModalOpen, (open) => {
	if (open) {
		modal.value?.show()
	}
})

async function handleCreateGroup() {
	if (await createGroup()) {
		clearLibraryInstanceSelection()
		modal.value?.hide()
	}
}
</script>
