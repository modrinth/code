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
			<span class="text-2xl font-semibold text-contrast">Create group</span>
		</template>

		<div class="flex flex-col gap-2.5 p-6">
			<label for="new-group-name" class="font-semibold text-contrast">Group name</label>
			<StyledInput
				id="new-group-name"
				ref="groupNameInput"
				v-model="newGroupName"
				placeholder="Enter group name"
				:maxlength="32"
			/>
			<span v-if="newGroupNameExists" class="text-sm font-medium text-red">
				A group with this name already exists.
			</span>
		</div>

		<div class="h-px bg-divider" />

		<div class="flex h-[400px] flex-col gap-3 overflow-y-auto bg-surface-2 py-4">
			<div class="px-6">
				<StyledInput
					v-model="newGroupSearch"
					:icon="SearchIcon"
					placeholder="Search instance"
					class="w-full"
				/>
			</div>

			<div
				v-if="newGroupInstances.length === 0"
				class="flex items-center justify-center py-12 text-secondary"
			>
				No instances found
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
							:src="instance.icon_path ? convertFileSrc(instance.icon_path) : undefined"
							:tint-by="instance.id"
							:alt="instance.name"
							size="2rem"
							rounded="md"
						/>
						<div class="flex min-w-0 items-center gap-2">
							<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
							<TagItem class="shrink-0">
								{{ instance.groups[0] ?? 'Ungrouped' }}
							</TagItem>
						</div>
					</div>
					<ButtonStyled
						:type="selectedNewGroupInstanceIds.has(instance.id) ? 'outlined' : 'standard'"
					>
						<button @click="toggleNewGroupInstance(instance.id)">
							<CheckIcon v-if="selectedNewGroupInstanceIds.has(instance.id)" />
							{{
								selectedNewGroupInstanceIds.has(instance.id)
									? 'Added'
									: instance.groups.length > 0
										? 'Move'
										: 'Add'
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="modal?.hide()">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canCreateGroup" @click="handleCreateGroup">
						<SpinnerIcon v-if="creatingGroup" class="animate-spin" />
						<PlusIcon v-else />
						Create group
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	PlusIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { Avatar, ButtonStyled, NewModal, StyledInput, TagItem } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { nextTick, ref, watch } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'

const {
	isNewGroupModalOpen,
	newGroupName,
	newGroupSearch,
	selectedNewGroupInstanceIds,
	creatingGroup,
	newGroupNameExists,
	newGroupInstances,
	canCreateGroup,
	closeNewGroupModal,
	toggleNewGroupInstance,
	createGroup,
} = useLibrary()

const modal = ref<InstanceType<typeof NewModal>>()
const groupNameInput = ref<InstanceType<typeof StyledInput>>()

watch(isNewGroupModalOpen, (open) => {
	if (open) {
		modal.value?.show()
		nextTick(() => {
			setTimeout(() => {
				groupNameInput.value?.select()
			}, 100)
		})
	}
})

async function handleCreateGroup() {
	if (await createGroup()) {
		modal.value?.hide()
	}
}
</script>
