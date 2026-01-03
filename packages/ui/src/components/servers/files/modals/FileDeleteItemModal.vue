<template>
	<NewModal ref="modal" fade="danger" :header="`Deleting ${item?.type}`">
		<form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleSubmit">
			<div
				class="relative flex w-full items-center gap-2 rounded-2xl border border-solid border-brand-red bg-bg-red p-6 shadow-md"
			>
				<div
					class="flex h-9 w-9 items-center justify-center rounded-full bg-highlight-red p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
				>
					<FolderOpenIcon v-if="item?.type === 'directory'" class="h-5 w-5" />
					<FileIcon v-else-if="item?.type === 'file'" class="h-5 w-5" />
				</div>
				<div class="flex flex-col">
					<span class="font-bold group-hover:text-contrast">{{ item?.name }}</span>
					<span
						v-if="item?.type === 'directory'"
						class="text-xs text-secondary group-hover:text-primary"
					>
						{{ item?.count }} items
					</span>
					<span v-else class="text-xs text-secondary group-hover:text-primary">
						{{ ((item?.size ?? 0) / 1024 / 1024).toFixed(2) }} MB
					</span>
				</div>
			</div>
			<div class="flex justify-start gap-4">
				<ButtonStyled color="red">
					<button type="submit">
						<TrashIcon class="h-5 w-5" />
						Delete {{ item?.type }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button type="button" @click="hide">
						<XIcon class="h-5 w-5" />
						Cancel
					</button>
				</ButtonStyled>
			</div>
		</form>
	</NewModal>
</template>

<script setup lang="ts">
import { FileIcon, FolderOpenIcon, TrashIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, NewModal } from '@modrinth/ui'
import { ref } from 'vue'

defineProps<{
	item: {
		name: string
		type: string
		count?: number
		size?: number
	} | null
}>()

const emit = defineEmits<{
	delete: []
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const handleSubmit = () => {
	emit('delete')
	hide()
}

const show = () => {
	modal.value?.show()
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
