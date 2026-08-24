<template>
	<NewModal
		ref="modal"
		no-padding
		scrollable
		actions-divider
		max-width="560px"
		width="560px"
		:on-hide="closeGroupInstancesModal"
	>
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{
					formatMessage(messages.title, {
						groupName:
							groupInstancesModalGroup?.id === 'group:none'
								? formatMessage(messages.ungrouped)
								: (groupInstancesModalGroup?.name ?? ''),
					})
				}}
			</span>
		</template>

		<div class="flex h-[400px] flex-col gap-3 overflow-y-auto bg-surface-2 py-4">
			<div class="px-6">
				<StyledInput
					v-model="groupInstancesSearch"
					:icon="SearchIcon"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					class="w-full"
				/>
			</div>

			<div
				v-if="groupInstances.length === 0"
				class="flex items-center justify-center py-12 text-secondary"
			>
				{{ formatMessage(messages.noInstancesFound) }}
			</div>
			<div v-else class="flex flex-col gap-1">
				<div
					v-for="instance in groupInstances"
					:key="instance.id"
					class="flex items-center justify-between gap-4 px-6 py-1.5 hover:bg-surface-3"
					:class="{ 'opacity-60': selectedGroupInstanceIds.has(instance.id) }"
				>
					<div class="flex min-w-0 items-center gap-2.5">
						<Avatar
							:src="getInstanceIconUrl(instance.icon_path)"
							:tint-by="instance.id"
							:alt="instance.name"
							size="2rem"
							rounded="md"
						/>
						<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
					</div>
					<Button
						:type="selectedGroupInstanceIds.has(instance.id) ? 'outlined' : 'base'"
						:disabled="
							groupInstancesModalGroup?.id === 'group:none' &&
							selectedGroupInstanceIds.has(instance.id)
						"
						@click="toggleGroupInstance(instance.id)"
					>
						<CheckIcon v-if="selectedGroupInstanceIds.has(instance.id)" />
						{{
							formatMessage(
								selectedGroupInstanceIds.has(instance.id) ? messages.added : messages.add,
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
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="savingGroupInstances" @click="save">
					<SpinnerIcon v-if="savingGroupInstances" class="animate-spin" />
					<CheckIcon v-else />
					{{ formatMessage(commonMessages.saveChangesButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'
import { getInstanceIconUrl } from '@/helpers/instance'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	ungrouped: {
		id: 'app.library.group.ungrouped',
		defaultMessage: 'Ungrouped',
	},
	title: {
		id: 'app.library.group.instances-modal.title',
		defaultMessage: 'Add instances to "{groupName}"',
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
})

const {
	isGroupInstancesModalOpen,
	groupInstancesModalGroup,
	groupInstancesSearch,
	groupInstances,
	selectedGroupInstanceIds,
	savingGroupInstances,
	closeGroupInstancesModal,
	toggleGroupInstance,
	saveGroupInstances,
} = useLibrary()

const modal = ref<InstanceType<typeof NewModal>>()

watch(isGroupInstancesModalOpen, (open) => {
	if (open) {
		modal.value?.show()
	}
})

async function save() {
	if (await saveGroupInstances()) {
		modal.value?.hide()
	}
}
</script>
