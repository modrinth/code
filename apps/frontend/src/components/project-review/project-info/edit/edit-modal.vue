<template>
	<NewModal
		ref="modal"
		:header="`Edit ${section}`"
		:width="width"
		:disable-close="saving"
		:on-hide="() => emit('cancel')"
		no-padding
		actions-divider
		scrollable
	>
		<div class="p-6">
			<slot />
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="quiet" :disabled="saving" @click="modal?.hide()"> Cancel </Button>
				<Button
					type="colored"
					color="brand"
					:loading="saving"
					:disabled="!canSave"
					@click="emit('save')"
				>
					Save project
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { Button, NewModal } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

withDefaults(
	defineProps<{
		section: string
		saving: boolean
		canSave: boolean
		width?: string
	}>(),
	{ width: '48rem' },
)

const emit = defineEmits<{
	cancel: []
	save: []
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')

defineExpose({
	show: () => modal.value?.show(),
	hide: () => modal.value?.hide(),
})
</script>
