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
					<ButtonStyled circular type="highlight-colored-text" :color="buttonColors[type]">
						<button aria-label="Toggle" @click.stop="expanded = !expanded">
							<ChevronDownIcon
								class="h-4 w-4 transition-transform duration-300"
								:class="expanded && 'rotate-180'"
							/>
						</button>
					</ButtonStyled>
					<ButtonStyled
						v-if="dismissible"
						circular
						type="highlight-colored-text"
						:color="buttonColors[type]"
					>
						<button aria-label="Dismiss" @click.stop="handleDismiss">
							<XIcon class="h-4 w-4" />
						</button>
					</ButtonStyled>
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

import ButtonStyled from './ButtonStyled.vue'

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
	border-color: rgba(255, 73, 110, 0.6);
}

.collapsible-admonition[data-type='critical'] .collapsible-admonition__item {
	background: rgba(255, 73, 110, 0.1);
}

.collapsible-admonition[data-type='critical'] .collapsible-admonition__item--bordered {
	border-top: 1px solid rgba(255, 73, 110, 0.6);
}

.collapsible-admonition[data-type='info'] {
	border-color: rgba(47, 158, 255, 0.6);
}

.collapsible-admonition[data-type='info'] .collapsible-admonition__item {
	background: rgba(47, 158, 255, 0.1);
}

.collapsible-admonition[data-type='info'] .collapsible-admonition__item--bordered {
	border-top: 1px solid rgba(47, 158, 255, 0.6);
}

.collapsible-admonition[data-type='warning'] {
	border-color: rgba(255, 163, 71, 0.6);
}

.collapsible-admonition[data-type='warning'] .collapsible-admonition__item {
	background: rgba(255, 163, 71, 0.1);
}

.collapsible-admonition[data-type='warning'] .collapsible-admonition__item--bordered {
	border-top: 1px solid rgba(255, 163, 71, 0.6);
}

.collapsible-admonition[data-type='success'] {
	border-color: rgba(27, 217, 106, 0.6);
}

.collapsible-admonition[data-type='success'] .collapsible-admonition__item {
	background: rgba(27, 217, 106, 0.1);
}

.collapsible-admonition[data-type='success'] .collapsible-admonition__item--bordered {
	border-top: 1px solid rgba(27, 217, 106, 0.6);
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
