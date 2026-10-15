<script setup lang="ts">
import { CopyCode } from '@modrinth/ui'
import { shallowRef, useTemplateRef, watch } from 'vue'

const props = defineProps<{
	html: string
}>()

const content = useTemplateRef<HTMLElement>('content')
const copies = shallowRef<{ target: Element; text: string }[]>([])

watch(
	[content, () => props.html],
	() => {
		copies.value = Array.from(
			content.value?.querySelectorAll('inline-copy-code') ?? [],
			(target) => {
				const text = target.textContent ?? ''
				target.replaceChildren()
				return { target, text }
			},
		)
	},
	{ flush: 'post' },
)
</script>

<template>
	<div ref="content" v-html="html" />
	<Teleport v-for="(copy, index) in copies" :key="index" :to="copy.target">
		<CopyCode
			:text="copy.text"
			class="max-w-full !rounded-[var(--radius-sm)] !bg-[var(--color-button-bg)] !px-1 !py-0 align-baseline !text-[80%] [&>span]:min-w-0 [&>span]:[overflow-wrap:anywhere] [&>svg]:shrink-0"
		/>
	</Teleport>
</template>
