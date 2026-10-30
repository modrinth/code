<script setup lang="ts">
import { SettingsFormGroup, Textarea } from '@modrinth/ui'
import { useId } from 'vue'

defineProps<{
	variant?: 'settings' | 'review'
	title: string
	titleFor?: string
	optional?: boolean
	disabled?: boolean
	placeholder?: string
	resize?: 'none' | 'vertical' | 'both'
}>()
const model = defineModel<string | undefined>()
const inputId = useId()
</script>

<template>
	<SettingsFormGroup :title="title" :title-for="titleFor ?? inputId" :optional="optional">
		<Textarea
			:id="titleFor ?? inputId"
			v-model="model"
			:rows="3"
			:resize="resize"
			:class="variant === 'review' ? 'w-full' : 'max-w-[40rem]'"
			:disabled="disabled"
			:placeholder="placeholder"
		/>
	</SettingsFormGroup>
</template>
