<script setup lang="ts">
import { CopyCode, ImageViewerEditor } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { parseIssueMessage } from './message-blocks'

const props = defineProps<{
	message: string
	highlighted?: boolean
}>()

const blocks = computed(() => parseIssueMessage(props.message, props.highlighted))
const images = computed(() =>
	blocks.value.flatMap((block) => (block.type === 'images' ? block.images : [])),
)
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
</script>

<template>
	<div class="markdown-body min-w-0">
		<template v-for="(block, index) in blocks" :key="index">
			<div v-if="block.type === 'markdown'" v-html="block.html" />
			<CopyCode
				v-else-if="block.type === 'copy'"
				:text="block.text"
				class="mb-4 max-w-full whitespace-pre-wrap text-left [overflow-wrap:anywhere] [&>span]:min-w-0 [&>svg]:shrink-0"
			/>
			<div v-else class="flex flex-wrap gap-3">
				<div v-for="image in block.images" :key="image.id" class="mb-4 w-[140px] max-w-full">
					<button
						type="button"
						class="block w-full cursor-zoom-in rounded-lg border-0 bg-surface-3 p-2"
						:aria-label="image.alt || image.src"
						@click="viewer?.show(images.findIndex((item) => item.id === image.id))"
					>
						<img
							:src="image.src"
							:alt="image.alt"
							loading="lazy"
							class="block h-24 w-full object-contain"
						/>
					</button>
					<span v-if="image.title" class="mt-1.5 block text-sm [overflow-wrap:anywhere]">{{
						image.title
					}}</span>
				</div>
			</div>
		</template>
		<ImageViewerEditor :key="message" ref="viewer" :items="images" editor="disabled" />
	</div>
</template>
