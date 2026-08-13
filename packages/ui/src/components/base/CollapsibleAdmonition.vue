<template>
	<Transition name="collapsible-admonition">
		<div
			v-if="!dismissed"
			:data-type="type"
			class="collapsible-admonition flex flex-col rounded-2xl border border-solid text-contrast overflow-hidden"
		>
			<div
				class="flex w-full cursor-pointer items-center gap-6 p-4"
				:class="headerBgClasses[type]"
				@click="expanded = !expanded"
			>
				<div class="flex flex-1 items-center gap-3">
					<TriangleAlertIcon :class="['h-5 w-5 flex-none', iconClasses[type]]" />
					<span class="text-base font-semibold text-contrast">
						<slot name="header">{{ header }}</slot>
					</span>
				</div>
				<div class="flex items-center gap-2">
					<IconButton
						type="quiet"
						:color="buttonColors[type]"
						label="Toggle"
						@click.stop="expanded = !expanded"
					>
						<ChevronDownIcon
							class="h-4 w-4 transition-transform duration-300"
							:class="expanded && 'rotate-180'"
						/>
					</IconButton>
					<IconButton
						v-if="dismissible"
						type="quiet"
						:color="buttonColors[type]"
						label="Dismiss"
						@click.stop="handleDismiss"
					>
						<XIcon class="h-4 w-4" />
					</IconButton>
				</div>
			</div>

			<div
				class="grid transition-[grid-template-rows] duration-300 ease-in-out"
				:class="expanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
			>
				<div class="overflow-hidden">
					<slot>
						<div
							v-for="(item, index) in items"
							:key="index"
							class="collapsible-admonition__item collapsible-admonition__item--bordered flex flex-col gap-1 p-4"
						>
							<p class="m-0 text-base font-semibold text-contrast">
								{{ item.title }}
							</p>
							<div
								v-for="(desc, di) in item.descriptions"
								:key="di"
								class="flex items-start gap-1.5"
							>
								<LightBulbIcon :class="['mt-0.5 h-5 w-5 flex-none', iconClasses[type]]" />
								<span class="text-base text-contrast/85">{{ desc }}</span>
							</div>
						</div>
					</slot>
				</div>
			</div>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { ChevronDownIcon, LightBulbIcon, TriangleAlertIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { IconButton } from '#ui/components/base/buttons'

export interface CollapsibleAdmonitionItem {
	title: string
	descriptions?: string[]
}

withDefaults(
	defineProps<{
		type?: 'info' | 'warning' | 'critical' | 'success'
		header?: string
		items?: CollapsibleAdmonitionItem[]
		dismissible?: boolean
	}>(),
	{
		type: 'critical',
		header: '',
		items: () => [],
		dismissible: false,
	},
)

const emit = defineEmits<{
	dismiss: []
}>()

const expanded = defineModel<boolean>({ default: false })
const dismissed = ref(false)

function handleDismiss() {
	dismissed.value = true
	emit('dismiss')
}

const headerBgClasses = {
	info: 'bg-bg-blue',
	warning: 'bg-bg-orange',
	critical: 'bg-bg-red',
	success: 'bg-bg-green',
}

const iconClasses = {
	info: 'text-brand-blue',
	warning: 'text-brand-orange',
	critical: 'text-brand-red',
	success: 'text-brand-green',
}

const buttonColors: Record<string, 'blue' | 'orange' | 'red' | 'green'> = {
	info: 'blue',
	warning: 'orange',
	critical: 'red',
	success: 'green',
}
</script>

<style scoped>
.collapsible-admonition[data-type='critical'] {
	border-color: color-mix(in srgb, var(--color-red) 60%, transparent);
}

.collapsible-admonition[data-type='critical'] .collapsible-admonition__item {
	background: var(--color-red-bg);
}

.collapsible-admonition[data-type='critical'] .collapsible-admonition__item--bordered {
	border-top: 1px solid color-mix(in srgb, var(--color-red) 60%, transparent);
}

.collapsible-admonition[data-type='info'] {
	border-color: color-mix(in srgb, var(--color-blue) 60%, transparent);
}

.collapsible-admonition[data-type='info'] .collapsible-admonition__item {
	background: var(--color-blue-bg);
}

.collapsible-admonition[data-type='info'] .collapsible-admonition__item--bordered {
	border-top: 1px solid color-mix(in srgb, var(--color-blue) 60%, transparent);
}

.collapsible-admonition[data-type='warning'] {
	border-color: color-mix(in srgb, var(--color-orange) 60%, transparent);
}

.collapsible-admonition[data-type='warning'] .collapsible-admonition__item {
	background: var(--color-orange-bg);
}

.collapsible-admonition[data-type='warning'] .collapsible-admonition__item--bordered {
	border-top: 1px solid color-mix(in srgb, var(--color-orange) 60%, transparent);
}

.collapsible-admonition[data-type='success'] {
	border-color: color-mix(in srgb, var(--color-green) 60%, transparent);
}

.collapsible-admonition[data-type='success'] .collapsible-admonition__item {
	background: var(--color-green-bg);
}

.collapsible-admonition[data-type='success'] .collapsible-admonition__item--bordered {
	border-top: 1px solid color-mix(in srgb, var(--color-green) 60%, transparent);
}

.collapsible-admonition-enter-active,
.collapsible-admonition-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.collapsible-admonition-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.collapsible-admonition-leave-to {
	opacity: 0;
	transform: translateY(-10px);
}
</style>
