<script lang="ts">
	import { classCombine } from '$lib/utils/classCombine'

	export let placeholder = ''
	/** A Svelte component */
	export let icon: any = undefined
	export let value = ''
	export let multiline = false
	/** An ID for better accessibility */
	export let id: string = undefined
	export let fill = false
	export let raised = false
	export let autofocus = false
</script>

<div class={classCombine(['text-input', raised && 'text-input--raised'])} class:fill>
	{#if multiline}
		<textarea {id} {placeholder} {autofocus} bind:value />
	{:else}
		<input type="text" {id} {placeholder} {autofocus} bind:value class:has-icon={icon} />
		{#if icon}
			<svelte:component this={icon} />
		{/if}
	{/if}
</div>

<style lang="postcss">
	.text-input {
		display: flex;
		flex-direction: column;
		grid-gap: 0.5rem;
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
		}

		&--raised > input,
		&--raised > textarea {
			border: 2px solid var(--color-text-lightest);
			box-shadow: var(--shadow-inset-sm), var(--shadow-raised);
		}

		input {
			padding: 0.25rem 1rem;

			&.has-icon {
				padding-left: 2.5rem;
			}
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
