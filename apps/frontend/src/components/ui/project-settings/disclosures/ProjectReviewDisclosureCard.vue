<script setup lang="ts">
import { Button, Toggle } from '@modrinth/ui'
import { type Component, computed, useId } from 'vue'

import EditButton from '~/components/project-review/project-info/edit/button.vue'

const props = defineProps<{
	title: string
	icon?: Component
	description?: string
	disabled?: boolean
	toggleDisabled?: boolean
	editing?: boolean
	onToggleEditing?: () => void
	onCancelEditing?: () => void
	onSaveEditing?: () => Promise<void>
	sectionCanSave?: boolean
	sectionSaving?: boolean
	sectionSaveError?: boolean
	sectionSaveDisabledReason?: string[]
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
			<div class="flex items-center justify-between gap-2.5">
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
				<slot name="lock-controls" />
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
		</div>
		<div
			v-if="$slots.expanded"
			v-show="enabled"
			class="disclosure-details editable-review-section relative flex flex-col gap-4 border-0 border-t border-solid border-surface-4 bg-surface-2 p-4 text-sm text-secondary"
			:class="{ 'box-border min-h-14': onToggleEditing }"
		>
			<div v-if="editing" class="absolute right-4 top-2 flex justify-end gap-1.5">
				<Button type="quiet" size="xs" :disabled="disabled" @click.stop="onCancelEditing?.()">
					Cancel
				</Button>
				<Button
					type="colored"
					color="brand"
					size="xs"
					:loading="sectionSaving"
					:disabled="disabled || !sectionCanSave"
					@click.stop="onSaveEditing?.()"
				>
					Save
				</Button>
			</div>
			<div v-if="onToggleEditing && !editing" class="absolute right-4 top-4 flex">
				<EditButton :section="title" :disabled="disabled" @click="onToggleEditing" />
			</div>
			<div class="flex min-w-0 flex-col gap-4" :class="{ 'pr-8': onToggleEditing && !editing }">
				<slot name="expanded" />
			</div>
			<template v-if="editing">
				<p v-if="sectionSaveError" role="alert" class="m-0 text-red">
					Could not save all disclosure changes. Your edits have been kept. Try saving again.
				</p>
				<p v-if="sectionSaveDisabledReason?.length" role="alert" class="m-0 text-orange">
					{{ sectionSaveDisabledReason.join(' ') }}
				</p>
			</template>
		</div>
	</section>
</template>

<style scoped>
.disclosure-details :deep(label.font-semibold.text-contrast),
.disclosure-details :deep(p.font-semibold.text-contrast) {
	@apply text-sm text-secondary;
}
</style>
