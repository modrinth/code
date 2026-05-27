<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
	(e: 'select'): void
	(e: 'edit', event: MouseEvent): void
}>()

const props = withDefaults(
	defineProps<{
		forwardImageSrc?: string
		backwardImageSrc?: string
		selected: boolean
		tooltip?: string
	}>(),
	{
		forwardImageSrc: undefined,
		backwardImageSrc: undefined,
		tooltip: undefined,
	},
)

const imagesLoaded = ref({
	forward: Boolean(props.forwardImageSrc),
	backward: Boolean(props.backwardImageSrc),
})

function onImageLoad(type: 'forward' | 'backward') {
	imagesLoaded.value[type] = true
}
</script>

<template>
	<div
		v-tooltip="tooltip ?? undefined"
		class="skin-button group relative flex items-end justify-center overflow-clip border border-solid transition-[background,border-color,box-shadow] duration-200 focus-within:outline focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-brand"
		:class="[
			selected ? 'skin-button--selected' : '',
			{ 'skin-button--with-actions': $slots['overlay-buttons'] },
		]"
	>
		<button
			class="absolute inset-0 z-10 cursor-pointer border-none bg-transparent p-0"
			:aria-label="tooltip ? `Select ${tooltip}` : 'Select skin'"
			:aria-pressed="selected"
			@click="emit('select')"
		></button>

		<div
			v-if="!(imagesLoaded.forward && imagesLoaded.backward)"
			class="skeleton-loader h-full w-full"
		>
			<div class="skeleton absolute inset-0 aspect-[5/7]"></div>
		</div>

		<span
			v-show="imagesLoaded.forward && imagesLoaded.backward"
			:class="[
				'skin-button__image-parent pointer-events-none relative z-0 mb-[1.5px] grid [transform-style:preserve-3d] place-items-stretch transition-transform duration-500 group-hover:[transform:rotateY(180deg)] with-shadow',
			]"
		>
			<img
				alt=""
				:src="forwardImageSrc"
				class="skin-button__image-facing col-start-1 row-start-1 h-full w-full object-contain [backface-visibility:hidden]"
				height="504"
				@load="onImageLoad('forward')"
			/>
			<img
				alt=""
				:src="backwardImageSrc"
				class="skin-button__image-away col-start-1 row-start-1 h-full w-full object-contain [backface-visibility:hidden] [transform:rotateY(180deg)]"
				height="504"
				@load="onImageLoad('backward')"
			/>
		</span>

		<span
			v-if="$slots['overlay-buttons']"
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
	background: linear-gradient(
		90deg,
		var(--color-bg) 25%,
		var(--color-raised-bg) 50%,
		var(--color-bg) 75%
	);
	background-size: 200% 100%;
	animation: wave 1500ms infinite linear;
}

@keyframes wave {
	0% {
		background-position: -200% 0;
	}
	100% {
		background-position: 200% 0;
	}
}

.skin-button {
	aspect-ratio: 31 / 40;
	border-color: var(--surface-4);
	border-radius: 20px;
	background: var(--surface-3);
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
	background: linear-gradient(
		180deg,
		rgba(0, 0, 0, 0) 0%,
		rgba(37, 39, 45, 0.2) 100%
	);
}

.skin-button:hover,
.skin-button:focus-within,
.skin-button--with-actions:hover,
.skin-button--with-actions:focus-within {
	border-color: var(--surface-5);
	background: var(--surface-4);
	box-shadow:
		0 1px 2px rgba(0, 0, 0, 0.25),
		0 1px 4px rgba(0, 0, 0, 0.15);
}

.skin-button--selected,
.skin-button--selected:hover,
.skin-button--selected:focus-within {
	border-color: var(--color-brand);
	background: var(--color-brand-highlight);
}

.skin-button__image-parent {
	width: 100%;
	height: 80%;
}

.with-shadow img {
	filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.4));
}

.skin-button__image-parent img {
	transition: filter 200ms ease-in-out;
}

.group:hover .skin-button__image-parent img {
	filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
}
</style>
