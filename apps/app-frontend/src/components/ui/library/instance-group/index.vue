<script setup lang="ts">
import { DropdownIcon, TrashIcon, XIcon } from '@modrinth/assets'
import {
	Accordion,
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewModal,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import Instance from '@/components/ui/library/instance-group/instance.vue'
import { useLibrary } from '@/components/ui/library/use-library'
import type {
	InstanceCard,
	InstanceGroup as InstanceGroupType,
} from '@/components/ui/library/use-library'

const props = defineProps<{
	instanceGroup: InstanceGroupType
}>()

const { formatMessage } = useVIntl()
const { isSectionCollapsed, setSectionCollapsed, deleteGroup, handleInstanceContextMenu } =
	useLibrary()

const instanceComponents = ref<InstanceCard[]>([])
const groupAccordion = ref<InstanceType<typeof Accordion>>()
const confirmDeleteGroupModal = ref<InstanceType<typeof NewModal>>()
const deletingGroup = ref(false)

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

function toggleGroup() {
	if (groupAccordion.value?.isOpen) {
		groupAccordion.value.close()
	} else {
		groupAccordion.value?.open()
	}
}
</script>

<template>
	<div class="instance-group relative">
		<div
			v-if="instanceGroup.key !== 'None'"
			class="group/header mb-3 flex w-full items-center gap-2 border-0 border-b border-solid border-b-surface-5 py-2.5"
		>
			<button
				class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 border-0 bg-transparent p-0 text-left"
				type="button"
				@click="toggleGroup"
			>
				<DropdownIcon
					class="size-5 shrink-0 text-secondary transition-all duration-300 group-hover/header:text-primary"
					:class="{ 'rotate-180': groupAccordion?.isOpen }"
				/>
				<span
					class="text-base font-semibold text-primary transition-colors group-hover/header:text-contrast"
				>
					{{ instanceGroup.key }}
				</span>
				<TagItem class="shrink-0 border-surface-3 bg-surface-2">
					{{ instanceGroup.instances.length }}
				</TagItem>
			</button>
			<ButtonStyled circular type="transparent">
				<button
					v-tooltip="formatMessage(messages.deleteGroup)"
					class="opacity-0 !transition-all duration-150 group-hover/header:opacity-100 -m-1.5"
					type="button"
					:aria-label="formatMessage(messages.deleteGroup)"
					:disabled="deletingGroup"
					@click="requestGroupDeletion"
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
				class="grid w-full grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-3 overflow-y-auto scroll-smooth"
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
