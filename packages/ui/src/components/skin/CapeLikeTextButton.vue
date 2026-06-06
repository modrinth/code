<script setup lang="ts">
const emit = defineEmits<{
	(e: 'click'): void
}>()

withDefaults(
	defineProps<{
		tooltip?: string
		highlighted?: boolean
	}>(),
	{
		tooltip: undefined,
		highlighted: false,
	},
)
</script>

<template>
	<button
		v-tooltip="tooltip"
		type="button"
		class="cape-like-text-button group m-0 block cursor-pointer border-0 bg-transparent p-0"
		:aria-label="tooltip"
		:aria-pressed="highlighted"
		@click="emit('click')"
	>
		<span
			:class="[
				'relative block overflow-hidden rounded-lg border-0 p-[3px] shadow-[var(--shadow-button)] transition-[transform,background,color,filter] duration-200 group-active:scale-95 group-hover:brightness-[--hover-brightness] group-focus-visible:brightness-[--hover-brightness]',
				highlighted ? 'bg-brand text-brand' : 'text-primary [background:var(--color-button-bg)]',
			]"
		>
			<span
				:class="[
					'relative z-10 block aspect-[10/16] min-h-[96px] w-[60px] overflow-hidden rounded-[5px]',
					highlighted
						? '[background:linear-gradient(var(--color-brand-highlight),var(--color-brand-highlight)),var(--color-button-bg)]'
						: '[background:var(--color-button-bg)]',
				]"
			>
				<span class="absolute inset-0 flex flex-col items-center justify-center text-center">
					<span class="mb-1 flex items-center justify-center leading-none">
						<slot name="icon"></slot>
					</span>
					<span class="block text-xs leading-none">
						<slot></slot>
					</span>
				</span>
			</span>
		</span>
	</button>
</template>
