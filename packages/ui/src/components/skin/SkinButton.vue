<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { ref, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	selectNamedSkin: { id: 'skins.select-named', defaultMessage: 'Select {name}' },
	selectSkin: { id: 'skins.select', defaultMessage: 'Select skin' },
})

const emit = defineEmits<{
	(e: 'select'): void
	(e: 'edit', event: MouseEvent): void
}>()

const props = withDefaults(
	defineProps<{
		forwardImageSrc?: string
		selected: boolean
		active?: boolean
		tooltip?: string
		disabled?: boolean
		isDragging?: boolean
	}>(),
	{
		forwardImageSrc: undefined,
		active: false,
		tooltip: undefined,
		disabled: false,
		isDragging: false,
	},
)

const imagesLoaded = ref({
	forward: false,
})

function onImageLoad() {
	imagesLoaded.value.forward = true
}

watch(
	() => props.forwardImageSrc,
	() => {
		imagesLoaded.value.forward = false
	},
)
</script>

<template>
	<div
		v-tooltip="tooltip ?? undefined"
		class="skin-button group relative flex items-end justify-center overflow-hidden border border-solid transition-[border-color,box-shadow] duration-200"
		:class="[
			selected ? 'skin-button--selected' : '',
			active ? 'skin-button--active' : '',
			{
				'skin-button--with-actions': $slots['overlay-buttons'] && !disabled,
				'skin-button--disabled': disabled,
				'skin-button--dragging': isDragging,
			},
		]"
	>
		<span
			v-if="$slots['top-buttons']"
			class="pointer-events-none absolute right-3 top-3 z-30 flex items-center gap-1"
		>
			<slot name="top-buttons" />
		</span>

		<button
			class="absolute inset-0 z-10 cursor-pointer border-none bg-transparent p-0 focus-visible:outline-none"
			:aria-label="
				tooltip
					? formatMessage(messages.selectNamedSkin, { name: tooltip })
					: formatMessage(messages.selectSkin)
			"
			:aria-pressed="selected"
			:disabled="disabled"
			@click="emit('select')"
		></button>

		<span
			v-if="active && !selected && !$slots['top-buttons']"
			class="pointer-events-none absolute right-3 top-3 z-20 size-3 rounded-full border-2 border-solid border-surface-3 bg-green"
		></span>

		<span
			v-if="selected"
			class="pointer-events-none absolute right-3 top-3 z-20 flex size-6 items-center justify-center rounded-full"
		>
			<span class="absolute inset-0 rounded-full bg-contrast"></span>
			<CheckIcon class="relative size-4 invert [stroke-width:3]" />
		</span>

		<div v-if="!imagesLoaded.forward" class="skeleton-loader h-full w-full">
			<div class="skeleton absolute inset-0 aspect-[5/7]"></div>
		</div>

		<span
			v-show="imagesLoaded.forward"
			:key="`${selected}-${active}`"
			:class="[
				'skin-button__image-parent pointer-events-none relative z-0 mb-[1.5px] grid place-items-stretch with-shadow',
			]"
		>
			<img
				alt=""
				:src="forwardImageSrc"
				class="skin-button__image-facing col-start-1 row-start-1 h-full w-full object-contain"
				height="504"
				@load="onImageLoad"
			/>
		</span>

		<span
			v-if="$slots['overlay-buttons'] && !disabled"
			class="pointer-events-none absolute inset-x-0 bottom-3 z-30 flex translate-y-2 items-center justify-start gap-1.5 px-3 opacity-0 transition-all duration-200 group-focus-within:translate-y-0 group-focus-within:opacity-100 group-hover:translate-y-0 group-hover:opacity-100"
		>
			<slot name="overlay-buttons" />
		</span>
	</div>
</template>

<style scoped lang="scss">
.skeleton-loader {
	aspect-ratio: 31 / 40;
}

.skeleton {
	background: linear-gradient(145deg, var(--surface-2), var(--surface-3));
	animation: skeleton-pulse 2700ms ease-in-out infinite;
}

@keyframes skeleton-pulse {
	0%,
	100% {
		filter: brightness(0.95);
	}
	50% {
		filter: brightness(1.15);
	}
}

@media (prefers-reduced-motion: reduce) {
	.skeleton {
		animation: none;
	}
}

.skin-button {
	aspect-ratio: 31 / 40;
	border-color: var(--surface-4);
	border-radius: 20px;
	background: var(--surface-3);
	isolation: isolate;
	box-shadow:
		0 1px 1px rgba(0, 0, 0, 0.25),
		0 1px 2px rgba(0, 0, 0, 0.15);
}

.skin-button::after {
	position: absolute;
	inset: 0;
	z-index: 5;
	pointer-events: none;
	content: '';
	background: linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, rgba(37, 39, 45, 0.2) 100%);
}

.skin-button:has(:focus-visible) {
	outline: 2px solid var(--color-brand);
	outline-offset: 2px;
}

.skin-button:not(.skin-button--disabled):hover,
.skin-button:not(.skin-button--disabled):focus-within,
.skin-button--with-actions:hover,
.skin-button--with-actions:focus-within {
	border-color: var(--surface-5);
	background: var(--surface-4);
	box-shadow:
		0 1px 2px rgba(0, 0, 0, 0.25),
		0 1px 4px rgba(0, 0, 0, 0.15);
}

.skin-button.skin-button--selected,
.skin-button.skin-button--selected:hover,
.skin-button.skin-button--selected:focus-within,
.skin-button.skin-button--selected.skin-button--with-actions:hover,
.skin-button.skin-button--selected.skin-button--with-actions:focus-within {
	border-color: color-mix(in srgb, var(--color-text-primary) 40%, transparent);
	background: var(--surface-3);
	filter: brightness(1.1);
}

.skin-button--disabled {
	opacity: 0.65;
}

.skin-button--disabled button {
	cursor: not-allowed;
}

.skin-button--dragging {
	pointer-events: none;
}

.skin-button__image-parent {
	width: 100%;
	height: 95%;
}

.skin-button__image-facing {
	transform: translateZ(0);
}

.with-shadow img {
	filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.4));
}

.skin-button__image-parent img {
	transition: filter 200ms ease-in-out;
}

.group:not(.skin-button--disabled):hover .skin-button__image-parent img {
	filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
}
</style>
