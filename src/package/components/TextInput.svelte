<script lang="ts">
	import type { SvelteComponent } from 'svelte'

	export let placeholder = ''
	export let icon: SvelteComponent = undefined
	export let value = ''
	export let multiline = false
	export let id: string = undefined
	export let fill = false
</script>

<div class="text-input" class:fill>
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
		gap: 0.5rem;
		position: relative;
		width: 20rem;

		&.fill {
			width: 100%;
			flex: 1;
		}

		input,
		textarea {
			border-radius: var(--rounded);
			box-shadow: var(--shadow-inset-sm);
			background-color: var(--color-button-bg);
			border: none;
			width: 100%;
			max-width: 100%;

			&.has-icon {
				padding-left: 2.5rem;
			}
		}

		input {
			padding: 0.25rem 1rem;
		}

		textarea {
			min-height: 2.5rem;
			padding: 0.5rem 1rem;
		}

		:global(.icon) {
			position: absolute;
			display: flex;
			height: 100%;
			left: 1rem;
			opacity: 0.75;
		}
	}
</style>
