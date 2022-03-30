<script lang="ts">
    // TODO: Move to TextInput.svelte & simplify options for text

    import { SvelteComponent } from 'svelte'

    export let key = ''
    export let label = ''
    export let placeholder = ''
    export let icon: string | SvelteComponent = ''
    export let type: 'text' | 'textarea'
    export let options: [
        {
            label: string;
            value: string | number;
            icon: string;
        }
    ] = []
    export let value = undefined

    export let wrap = false
</script>

{#if label}
    <div class="input__label">
        {label}
    </div>
{/if}
{#if type === 'text'}
    <input type="text" name={key} class="input-box" {placeholder} bind:value={value}/>
{:else if type === 'textarea'}
    <textarea name={key} class="input-box input-box--fill" {placeholder} bind:value={value}/>
{/if}

<style lang="postcss">
    .input-box {
        border-radius: var(--rounded-sm);
        box-shadow: var(--shadow-inset-sm);
        background-color: var(--color-button-bg);
        border: none;
        padding: 0.25rem 0.75rem;
        width: 20rem;
        max-width: 100%;

        &--fill {
            width: 100%;
            padding: 0.5rem 0.75rem;
        }
    }

    textarea {
        min-height: 2.5rem;
    }
</style>
