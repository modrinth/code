<script lang="ts">
	import { Button } from 'omorphia'
	import IconMoon from 'virtual:icons/heroicons-outline/moon'
	import IconSun from 'virtual:icons/heroicons-outline/sun'

	export let meta: { raised: boolean; column: boolean }

	let theme = 'light'
	let background = meta.raised ? 'var(--color-raised-bg)' : 'var(--color-bg)'
</script>

<div class="example">
	<div class="example__preview theme-{theme} base" class:column={meta.column} style:background>
		<slot name="example" />
	</div>
	<div class="example__source">
		<div class="example__source__options">
			<Button
				color="primary-light"
				on:click={() => (theme === 'light' ? (theme = 'dark') : (theme = 'light'))}>
				{#if theme === 'light'}
					<IconMoon />
				{:else}
					<IconSun />
				{/if}
			</Button>
		</div>
		<pre class="example__source__code language-svelte"><slot name="code" /></pre>
	</div>
</div>

<style lang="postcss">
	.example {
		margin: 1rem 0 2rem;

		&__preview {
			border-radius: var(--rounded-sm-top);
			border: solid 2px hsl(0, 0%, 20%);
			border-bottom: none;
			display: flex;
			gap: 1rem;
			flex-wrap: wrap;
			position: relative;
			justify-content: flex-start;
			z-index: 1;
			padding: 1rem;

			&.column {
				flex-direction: column;
			}
		}

		&__source {
			position: relative;

			&__options {
				position: absolute;
				top: 0;
				right: 0;
				padding: 0.5rem;
				display: flex;
				justify-content: flex-end;

				:global(button) {
					color: black;
				}
			}

			&__code {
				margin: 0;
				border-radius: var(--rounded-sm-bottom) !important;
				background: hsl(220, 13%, 22%);
			}
		}
	}
</style>
