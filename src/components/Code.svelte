<script lang="ts">
	import IconClipboardCopy from 'virtual:icons/heroicons-outline/clipboard-copy'
	import IconCheck from 'virtual:icons/heroicons-outline/check'

	export let text: string

	let copied = false
</script>

<button
	class="code"
	class:copied
	title="Copy code to clipboard"
	on:click={async () => {
		await navigator.clipboard.writeText(text)
		copied = true
	}}>
	{text}
	{#if copied}
		<IconCheck />
	{:else}
		<IconClipboardCopy />
	{/if}
</button>

<style lang="postcss">
	.code {
		display: flex;
		grid-gap: 0.5rem;
		font-family: var(--mono-font);
		padding: 0.25rem 0.5rem;
		background-color: var(--color-code-bg);
		width: min-content;
		border-radius: var(--rounded-sm);
		user-select: text;

		&.copied {
			cursor: default;
		}

		&:hover:not(.copied) {
			filter: brightness(0.9);
		}
	}
</style>
