<script setup lang="ts">
import { Toggle } from '@modrinth/ui'
import { type Component, computed, useId } from 'vue'

const props = defineProps<{
	title: string
	icon?: Component
	description?: string
	disabled?: boolean
	toggleDisabled?: boolean
}>()
const enabled = defineModel<boolean>({ required: true })
const titleId = useId()
const isToggleLocked = computed(() => !!props.disabled || (!!props.toggleDisabled && enabled.value))
</script>

<template>
	<section
		class="overflow-clip rounded-2xl border border-solid border-surface-3 bg-surface-2"
		:aria-labelledby="titleId"
	>
		<div class="flex flex-col gap-3 p-4">
			<div class="flex items-start justify-between gap-4">
				<div class="min-w-0 flex-1">
					<h3 :id="titleId" class="m-0 flex items-center gap-2 text-lg font-semibold text-contrast">
						<component
							:is="icon"
							v-if="icon"
							class="size-5 shrink-0 text-primary"
							aria-hidden="true"
						/>
						{{ title }}
					</h3>
				</div>
				<Toggle
					v-model="enabled"
					class="shrink-0"
					:disabled="isToggleLocked"
					:aria-labelledby="titleId"
				/>
			</div>
			<div v-if="$slots['updated-by'] || description || $slots.default" class="flex flex-col gap-2">
				<div v-if="$slots['updated-by']" class="text-sm">
					<slot name="updated-by" />
				</div>
				<div
					v-if="description || $slots.default"
					class="flex flex-col gap-2 leading-normal [&>p]:m-0"
				>
					<p v-if="description">{{ description }}</p>
					<slot />
				</div>
			</div>
			<div v-if="$slots['lock-controls']" class="ml-auto max-w-full overflow-x-auto">
				<slot name="lock-controls" />
			</div>
		</div>
		<div
			v-if="enabled && $slots.expanded"
			class="flex flex-col gap-4 border-0 border-t border-solid border-surface-4 bg-surface-2 p-4"
		>
			<slot name="expanded" />
		</div>
	</section>
</template>
