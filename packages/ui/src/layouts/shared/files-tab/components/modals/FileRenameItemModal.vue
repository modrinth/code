<template>
	<NewModal ref="modal" :header="`Rename ${item?.name}`" max-width="500px">
		<form class="space-y-6 md:min-w-[400px]" @submit.prevent="handleSubmit">
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">New name</span>
				<StyledInput ref="renameInput" v-model="itemName" wrapper-class="w-full" />
				<div v-if="submitted && error" class="text-sm text-red">{{ error }}</div>
			</label>
		</form>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="hide">
						<XIcon class="h-5 w-5" />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!!error && submitted" @click="handleSubmit">
						<EditIcon class="h-5 w-5" />
						Rename
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { EditIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import NewModal from '#ui/components/modal/NewModal.vue'

const props = defineProps<{
	item: { name: string; type: string } | null
}>()

const emit = defineEmits<{
	rename: [newName: string]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const renameInput = ref<HTMLInputElement | null>(null)
const itemName = ref('')
const submitted = ref(false)

const error = computed(() => {
	if (!itemName.value) {
		return 'Name is required.'
	}
	if (props.item?.type === 'file') {
		const validPattern = /^[a-zA-Z0-9-_.\s]+$/
		if (!validPattern.test(itemName.value)) {
			return 'Name must contain only alphanumeric characters, dashes, underscores, dots, or spaces.'
		}
	} else {
		const validPattern = /^[a-zA-Z0-9-_\s]+$/
		if (!validPattern.test(itemName.value)) {
			return 'Name must contain only alphanumeric characters, dashes, underscores, or spaces.'
		}
	}
	return ''
})

const handleSubmit = () => {
	submitted.value = true
	if (!error.value) {
		emit('rename', itemName.value)
		hide()
	}
}

const show = (item: { name: string; type: string }) => {
	itemName.value = item.name
	submitted.value = false
	modal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			renameInput.value?.focus()
		}, 100)
	})
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
