<script lang="ts">
	import type { SvelteComponent } from 'svelte'

	export let placeholder = ''
	export let icon: SvelteComponent = undefined
	export let value = ''
	export let multiline = false
	export let id: string = undefined
</script>

<div class="text-input">
	{#if multiline}
		<textarea name={id} {placeholder} bind:value />
	{:else}
		<input type="text" name={id} {placeholder} bind:value class:has-icon={icon} />
		{#if icon}
			<svelte:component this={icon} />
		{/if}
	{/if}
</div>

<style lang="postcss">
	.text-input {
		display: flex;
		flex-direction: column;
		gap: 8px;
		position: relative;

		input,
		textarea {
			border-radius: var(--rounded-sm);
			box-shadow: var(--shadow-inset-sm);
			background-color: var(--color-button-bg);
			border: none;
			padding: 6px 14px;
			width: 20rem;
			max-width: 100%;
		}

		textarea {
			min-height: 2.5rem;
		}

		:global(.icon) {
			position: absolute;
			display: flex;
			height: 100%;
			left: 14px;
			opacity: 0.75;
		}

		input.has-icon {
			padding-left: 40px;
		}
	}
</style>
