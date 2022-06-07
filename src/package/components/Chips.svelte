<script lang="ts">
	import Button from './Button.svelte'
	import IconCheck from 'virtual:icons/heroicons-outline/check'

	interface Option {
		label: string
		value: string | number
	}

	export let options: Option[] = []
	export let value: string | number
	// If set to true, one chip is always selected
	export let neverEmpty = false

	let selected: Option | null = options.find((option) => option.value === (value || ''))

	$: if (selected) {
		value = selected.value
	} else {
		value = ''
	}
</script>

<div class="chips">
	{#each options as option}
		{@const isSelected = selected?.value === option.value}
		<Button
			color={isSelected ? 'primary-light' : undefined}
			on:click={() => {
				isSelected && !neverEmpty ? (selected = null) : (selected = option)
			}}>
			{#if isSelected}
				<IconCheck />
			{/if}
			{option.label}
		</Button>
	{/each}
</div>

<style lang="postcss">
	.chips {
		display: flex;
		gap: 0.5rem;
	}
</style>
