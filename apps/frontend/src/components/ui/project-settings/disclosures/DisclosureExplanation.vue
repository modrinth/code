<script setup lang="ts">
import { Field, Textarea } from '@modrinth/ui'
import { useId } from 'vue'

defineProps<{
	variant?: 'settings' | 'review'
	title: string
	optional?: boolean
	disabled?: boolean
	placeholder?: string
	resize?: 'none' | 'vertical' | 'both'
}>()
const model = defineModel<string | undefined>()
const inputId = useId()
</script>

<template>
	<Field :label="title" :input-id="inputId" :label-optional="optional">
		<Textarea
			:id="inputId"
			v-model="model"
			:rows="3"
			:resize="resize"
			:class="variant === 'review' ? 'w-full' : 'max-w-[40rem]'"
			:disabled="disabled"
			:placeholder="placeholder"
		/>
	</Field>
</template>
