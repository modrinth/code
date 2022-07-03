<script lang="ts">
	// TODO: Fix mobile support, currently just cuts off buttons

	import IconArrowRight from 'virtual:icons/heroicons-outline/arrow-right'
	import IconArrowLeft from 'virtual:icons/heroicons-outline/arrow-left'
	import IconMinus from 'virtual:icons/heroicons-outline/minus'
	import Button from './Button.svelte'
	import { createEventDispatcher } from 'svelte'

	export let page: number
	export let count: number

	$: options =
		count > 4
			? page + 3 >= count
				? [1, '-', count - 4, count - 3, count - 2, count - 1, count]
				: page > 4
				? [1, '-', page - 1, page, page + 1, '-', count]
				: [1, 2, 3, 4, 5, '-', count]
			: Array.from({ length: count }, (_, i) => i + 1)

	const dispatch = createEventDispatcher()
</script>

{#if count > 1}
	<div class="pagination">
		<Button
			raised
			on:click={() => dispatch('change', page - 1)}
			disabled={page <= 1}
			title="Last page"
			><IconArrowLeft height="20px" />
		</Button>
		{#each options as option}
			{#if option === '-'}
				<IconMinus class="pagination__dash" />
			{:else}
				<Button
					color={option === page ? 'primary' : ''}
					raised
					on:click={() => dispatch('change', option)}>{option}</Button>
			{/if}
		{/each}
		<Button
			raised
			on:click={() => dispatch('change', page + 1)}
			disabled={page >= count}
			title="Next page">
			<IconArrowRight height="20px" />
		</Button>
	</div>
{/if}

<style lang="postcss">
	.pagination {
		align-self: center;
		display: flex;
		gap: 0.5rem;
		align-items: center;

		:global(.button) {
			box-shadow: var(--shadow-inset-sm), var(--shadow-raised);
		}

		:global(.icon) {
			width: 1rem;
			aspect-ratio: 1 / 1;
		}

		:global(&__dash) {
			margin: 0 0.5rem;
		}

		@media (width <= 500px) {
			gap: 0.25rem;

			:global(> *:nth-child(4)),
			:global(> *:nth-child(6)) {
				display: none;
			}
		}
	}
</style>
