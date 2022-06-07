<script lang="ts">
	import Checkbox from './Checkbox.svelte'
	import type { Option } from './types'

	export let value = []
	export let options: Option[] = []

	/** Wrap the options horizontally */
	export let wrap = false

	const handleChange = (e, key) => {
		if (e.target.checked) {
			if (!value) value = []
			value = [key, ...value]
		} else {
			value = value.filter((it) => key !== it)
		}
	}
</script>

<div class="checkbox-list" class:wrap>
	{#each options as option}
		<Checkbox on:change={(e) => handleChange(e, option.value)}>
			{#if option.icon && typeof option.icon === 'string'}
				{@html option.icon}
			{:else if option.icon}
				<svelte:component this={option.icon} />
			{/if}
			{option.label}
		</Checkbox>
	{/each}
</div>

<style lang="postcss">
	.checkbox-list {
		display: flex;
		flex-direction: column;
		gap: 2px;

		&.wrap {
			flex-direction: row;
			flex-wrap: wrap;
			gap: 2rem;
		}
	}
</style>
