<script setup lang="ts">
import { FloatingMenu } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { severityOrder } from '../helpers'
import type { ClassGroup, FlagItem } from '../types'
import TraceName from './name.vue'

const props = defineProps<{ flags: FlagItem[]; label: string }>()
const emit = defineEmits<{ select: [fileKey: string] }>()
const menu = ref<InstanceType<typeof FloatingMenu>>()

const files = computed(() => {
	const groups = new Map<string, ClassGroup>()
	for (const flag of props.flags) {
		const key = `${flag.detail.jar ?? ''}::${flag.detail.file_path}`
		let group = groups.get(key)
		if (!group) {
			group = { key, jar: flag.detail.jar ?? null, filePath: flag.detail.file_path, flags: [] }
			groups.set(key, group)
		}
		group.flags.push(flag)
	}
	for (const group of groups.values()) {
		group.flags.sort((a, b) => severityOrder[b.detail.severity] - severityOrder[a.detail.severity])
	}
	return [...groups.values()]
})

function select(fileKey: string) {
	menu.value?.hide()
	emit('select', fileKey)
}
</script>

<template>
	<FloatingMenu ref="menu" placement="bottom-end" :arrow="false" bare>
		<button
			type="button"
			class="custom-focus-indicator h-8 cursor-pointer whitespace-nowrap border-0 bg-transparent px-3 text-sm font-semibold text-secondary hover:bg-surface-4 hover:text-primary"
		>
			{{ label }}
		</button>
		<template #popper>
			<section
				aria-label="Remaining traces"
				class="review-scrollbars w-max max-w-[calc(100vw-2rem)] rounded-xl border border-solid border-surface-5 bg-surface-2 p-1.5 shadow-2xl"
			>
				<h4 class="m-0 px-3 pb-2 pt-1.5 text-xs font-medium text-secondary">
					{{ label }} · {{ files.length }} {{ files.length === 1 ? 'file' : 'files' }}
				</h4>
				<div
					class="grid max-h-[60vh] grid-cols-[minmax(0,max-content)_minmax(0,max-content)] gap-x-6 gap-y-2.5 overflow-y-auto overscroll-contain px-3"
				>
					<div
						v-for="file in files"
						:key="file.key"
						class="col-span-2 grid grid-cols-[subgrid] border-0 border-b border-solid border-surface-4 pb-2.5 last:border-b-0 last:pb-0"
					>
						<div class="min-w-0 py-1">
							<button
								type="button"
								class="custom-focus-indicator cursor-pointer break-all border-0 bg-transparent p-0 text-left text-sm font-medium text-primary underline-offset-2 hover:underline focus-visible:underline"
								@click="select(file.key)"
							>
								{{ file.filePath }}
							</button>
						</div>
						<div class="flex min-w-0 flex-col items-start gap-0.5">
							<div v-for="flag in file.flags" :key="flag.detail.id" class="flex w-full px-1.5 py-1">
								<TraceName :issue-type="flag.issueType" :severity="flag.detail.severity" colored />
							</div>
						</div>
					</div>
				</div>
			</section>
		</template>
	</FloatingMenu>
</template>
