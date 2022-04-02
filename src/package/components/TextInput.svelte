<script lang="ts">
    import type { SvelteComponent } from 'svelte'

    export let placeholder = ''
    export let icon: SvelteComponent = ''
    export let value = ''
    export let multiline = false
    export let id: string = undefined
</script>

<div class="text-input">
    {#if multiline}
        <textarea name={id} {placeholder} bind:value={value}/>
    {:else}
        <input type="text" name={id} {placeholder} bind:value={value}/>
    {/if}
</div>


<style lang="postcss">
    .text-input {
        display: flex;
        flex-direction: column;
        gap: 8px;

        input, textarea {
            border-radius: var(--rounded-sm);
            box-shadow: var(--shadow-inset-sm);
            background-color: var(--color-button-bg);
            border: none;
            padding: 0.25rem 0.75rem;
            width: 20rem;
            max-width: 100%;
        }

        &--fill {
            width: 100%;
            padding: 0.5rem 0.75rem;
        }


        textarea {
            min-height: 2.5rem;
        }
    }
</style>
