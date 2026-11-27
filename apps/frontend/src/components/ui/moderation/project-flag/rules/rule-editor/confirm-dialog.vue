<template>
	<dialog
		ref="dialog"
		:aria-labelledby="titleId"
		:aria-describedby="descriptionId"
		class="m-auto w-[calc(100%_-_2rem)] max-w-md rounded-2xl border border-solid border-surface-5 bg-surface-2 p-6 text-primary shadow-2xl backdrop:bg-black/50"
		@cancel.prevent="finish(false)"
	>
		<h2 :id="titleId" class="m-0 text-xl font-semibold text-contrast">{{ title }}</h2>
		<p :id="descriptionId" class="my-4">{{ description }}</p>
		<div class="flex flex-wrap justify-end gap-2">
			<Button type="outlined" autofocus @click="finish(false)">{{ cancelLabel }}</Button>
			<Button type="colored" color="red" @click="finish(true)">{{ confirmLabel }}</Button>
		</div>
	</dialog>
</template>

<script setup lang="ts">
import { Button } from '@modrinth/ui'
import { onBeforeUnmount, useId, useTemplateRef } from 'vue'

withDefaults(
	defineProps<{
		title: string
		description: string
		confirmLabel: string
		cancelLabel?: string
	}>(),
	{ cancelLabel: 'Cancel' },
)

const id = useId()
const titleId = `${id}-title`
const descriptionId = `${id}-description`
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
let pending: Promise<boolean> | undefined
let resolvePrompt: ((confirmed: boolean) => void) | undefined

function prompt(): Promise<boolean> {
	if (pending) return pending.promise
	const element = dialog.value
	if (!element?.isConnected) return Promise.resolve(false)
	const promise = new Promise<boolean>((resolve) => {
		resolvePrompt = resolve
	})
	pending = promise
	element.showModal()
	return promise
}

function finish(confirmed: boolean) {
	const resolve = resolvePrompt
	resolvePrompt = undefined
	pending = undefined
	dialog.value?.close()
	resolve?.(confirmed)
}

onBeforeUnmount(() => finish(false))
defineExpose({ prompt })
</script>
