<script setup lang="ts">
import { DropdownIcon, TrashIcon, XIcon } from '@modrinth/assets'
import {
	Accordion,
	ButtonStyled,
	commonMessages,
	defineMessages,
	InlineEditableText,
	NewModal,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import Instance from '@/components/ui/library/instance-group/instance.vue'
import type {
	InstanceCard,
	InstanceGroup as InstanceGroupType,
} from '@/components/ui/library/use-library'
import { useLibrary } from '@/components/ui/library/use-library'

const props = defineProps<{
	instanceGroup: InstanceGroupType
}>()

const { formatMessage } = useVIntl()
const {
	isSectionCollapsed,
	setSectionCollapsed,
	deleteGroup,
	isValidGroupName,
	renameGroup,
	handleInstanceContextMenu,
} = useLibrary()

const instanceComponents = ref<InstanceCard[]>([])
const groupAccordion = ref<InstanceType<typeof Accordion>>()
const groupOptions = ref<InstanceType<typeof ContextMenu>>()
const confirmDeleteGroupModal = ref<InstanceType<typeof NewModal>>()
const deletingGroup = ref(false)
const groupName = ref(props.instanceGroup.key)

const messages = defineMessages({
	deleteGroup: {
		id: 'app.library.group.delete',
		defaultMessage: 'Delete group',
	},
	deleteGroupDescription: {
		id: 'app.library.group.delete-description',
		defaultMessage: 'Instances in this group will be ungrouped.',
	},
})

function openInstanceContextMenu(event: MouseEvent, instanceId: string, instanceGroupName: string) {
	const instanceComponent = instanceComponents.value.find(
		(component) => component.instance.id === instanceId,
	)
	if (!instanceComponent) return

	handleInstanceContextMenu(event, instanceComponent, instanceGroupName)
}

async function removeGroup() {
	if (deletingGroup.value) return

	deletingGroup.value = true
	const deleted = await deleteGroup(props.instanceGroup.key)
	deletingGroup.value = false

	if (deleted) {
		confirmDeleteGroupModal.value?.hide()
	}
}

function requestGroupDeletion() {
	if (props.instanceGroup.instances.length > 0) {
		confirmDeleteGroupModal.value?.show()
	} else {
		void removeGroup()
	}
}

function openGroupContextMenu(event: MouseEvent) {
	groupOptions.value?.showMenu(event, props.instanceGroup, [
		{ name: 'delete_group', color: 'danger' },
	])
}

function handleGroupOption({ option }: { option: string }) {
	if (option === 'delete_group') {
		requestGroupDeletion()
	}
}

function toggleGroup() {
	if (groupAccordion.value?.isOpen) {
		groupAccordion.value.close()
	} else {
		groupAccordion.value?.open()
	}
}

function validateGroupName(value: string) {
	return isValidGroupName(value, props.instanceGroup.key)
}

async function updateGroupName(value: string) {
	return await renameGroup(props.instanceGroup.id, props.instanceGroup.key, value)
}

watch(
	() => props.instanceGroup.key,
	(value) => {
		groupName.value = value
	},
)
</script>

<template>
	<div class="instance-group relative">
		<div
			v-if="instanceGroup.key !== 'None'"
			class="group/header mb-3 flex w-full cursor-pointer items-center gap-2 border-0 border-b border-solid border-b-surface-5 py-2.5"
			@click="toggleGroup"
			@contextmenu.prevent.stop="openGroupContextMenu"
		>
			<button
				class="flex shrink-0 cursor-pointer items-center border-0 bg-transparent p-0"
				type="button"
				:aria-expanded="groupAccordion?.isOpen"
				:aria-label="groupAccordion?.isOpen ? 'Collapse group' : 'Expand group'"
				@click.stop="toggleGroup"
			>
				<DropdownIcon
					class="size-5 shrink-0 text-secondary transition-all duration-300 group-hover/header:text-primary"
					:class="{ 'rotate-180': groupAccordion?.isOpen }"
				/>
			</button>
			<InlineEditableText
				v-model="groupName"
				class="text-base font-semibold text-primary select-none group-hover/header:text-contrast"
				:edit-label="formatMessage(commonMessages.renameButton)"
				max-width="24rem"
				:max-length="32"
				:on-change="updateGroupName"
				:validate="validateGroupName"
				@click.stop
			/>
			<TagItem v-if="instanceGroup.instances.length" class="shrink-0 border-surface-3 bg-surface-2">
				{{ instanceGroup.instances.length }}
			</TagItem>
			<div class="min-w-0 flex-1" />
			<ButtonStyled circular type="transparent">
				<button
					v-tooltip="formatMessage(messages.deleteGroup)"
					class="opacity-0 !transition-all duration-150 group-hover/header:opacity-100 -m-1.5"
					type="button"
					:aria-label="formatMessage(messages.deleteGroup)"
					:disabled="deletingGroup"
					@click.stop="requestGroupDeletion"
				>
					<TrashIcon class="!size-4 !min-h-4 !min-w-4" />
				</button>
			</ButtonStyled>
		</div>
		<Accordion
			ref="groupAccordion"
			:open-by-default="!isSectionCollapsed(instanceGroup.key)"
			class="w-full"
			@on-open="setSectionCollapsed(instanceGroup.key, false)"
			@on-close="setSectionCollapsed(instanceGroup.key, true)"
		>
			<p
				v-if="instanceGroup.instances.length === 0"
				class="m-0 py-2.5 pl-0.5 text-base font-medium text-secondary"
			>
				No instances in this group.
			</p>
			<section
				v-else
				class="grid w-full grid-cols-[repeat(auto-fill,minmax(20rem,22rem))] gap-3 overflow-y-auto scroll-smooth"
			>
				<Instance
					v-for="instance in instanceGroup.instances"
					ref="instanceComponents"
					:key="instance.id + instance.install_stage"
					:instance="instance"
					@contextmenu.prevent.stop="
						(event: MouseEvent) => openInstanceContextMenu(event, instance.id, instanceGroup.key)
					"
				/>
			</section>
		</Accordion>
	</div>

	<ContextMenu ref="groupOptions" @option-clicked="handleGroupOption">
		<template #delete_group> <TrashIcon /> {{ formatMessage(messages.deleteGroup) }} </template>
	</ContextMenu>

	<NewModal
		ref="confirmDeleteGroupModal"
		:header="formatMessage(messages.deleteGroup)"
		fade="danger"
		width="500px"
	>
		<p class="m-0 text-base text-primary">
			{{ formatMessage(messages.deleteGroupDescription) }}
		</p>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button type="button" @click="confirmDeleteGroupModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button type="button" :disabled="deletingGroup" @click="removeGroup">
						<TrashIcon />
						{{ formatMessage(messages.deleteGroup) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>
