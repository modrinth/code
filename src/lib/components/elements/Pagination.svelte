<script lang="ts">
    // TODO: Modify to match new button props

    import IconArrowRight from 'virtual:icons/heroicons-outline/arrow-right'
    import IconArrowLeft from 'virtual:icons/heroicons-outline/arrow-left'
    import IconMinus from 'virtual:icons/heroicons-outline/minus'
    import Button from "./buttons/Button.svelte";
    import { createEventDispatcher } from 'svelte'

    export let page: number;
    export let count: number;

    $: options = count > 4
        ? page + 3 >= count
            ? [1, '-', count - 4, count - 3, count - 2, count - 1, count]
            : page > 4
                ? [1, '-', page - 1, page, page + 1, '-', count]
                : [1, 2, 3, 4, 5, '-', count]
        : Array.from({ length: count }, (_, i) => i + 1)

    const dispatch = createEventDispatcher();
</script>

{#if count > 1}
    <div class="pagination">
        <Button icon={IconArrowLeft} color="raised" on:click={() => dispatch('change', page - 1)} disabled={page <= 1} title="Last page"/>
        {#each options as option}
            {#if option === '-'}
                <IconMinus class="pagination__dash" />
            {:else}
                <Button label={option} color={option === page ? 'brand' : 'raised'} on:click={() => dispatch('change', option)} evenPadding={true} />
            {/if}
        {/each}
        <Button icon={IconArrowRight} color="raised" on:click={() => dispatch('change', page + 1)} disabled={page >= count} title="Next page" />
    </div>
{/if}

<style lang="postcss">
    .pagination {
        align-self: center;
        display: flex;
        grid-gap: 0.5rem;
        align-items: center;

        :global(&__dash) {
            margin: 0 0.5rem;
            width: 1rem;
            height: 1rem;
        }

        @media (width <= 500px) {
            grid-gap: 0.25rem;

            :global(> *:nth-child(4)), :global(> *:nth-child(6)) {
                display: none;
            }
        }
    }
</style>
