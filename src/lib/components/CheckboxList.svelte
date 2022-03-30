<script lang="ts">
    import Checkbox from './Checkbox.svelte'
    import type { SvelteComponent } from 'svelte'

    interface Option {
        label: string;
        /** The element that will be in the `value` array while the option is checked */
        value: string | number;
        icon: SvelteComponent;
    }

    export let value = []
    export let options: Option[] = []

    const handleChange = (e, key) => {
        if (e.target.checked) {
            if (!value) value = []
            value = [key, ...value]
        } else {
            value = value.filter((it) => key !== it)
        }
    }
</script>

<div class="checkbox-list">
    {#each options as option}
        <Checkbox on:change={(e) => handleChange(e, option.value)}>
            {#if option.icon}
                <svelte:component this={option.icon}/>
            {/if}
            {option.label}
        </Checkbox>
    {/each}
</div>

<style lang="postcss">
    .checkbox-list {
        display: flex;
        flex-direction: column;

        &.wrap {
            flex-direction: row;
            flex-wrap: wrap;
            grid-gap: 2rem;
        }
    }
</style>