<script lang="ts">
    import { classCombine } from '$lib/utils/classCombine'

    // The element to be styled as a button
    export let as: 'button' | 'a' | 'summary' | 'input' = 'button'
    export let href: string
    if (href) as = 'a'

    // Use `value` if the button is an `<input`>
    export let value: string

    export let size: 'sm' | 'md' | 'lg' = 'md'
    export let color: 'raised' | 'primary' | 'primary-light' | 'danger'| 'danger-light' | 'transparent'

    // Show notification badge in the upper right of button
    export let badge = false

    export let disabled = false

    let className: string
    $: className = classCombine(['button', `button--size-${size}`, `button--color-${color}`, badge && 'has-badge'])
</script>

{#if as === 'button'}
    <button class={className} {disabled}>
        <slot/>
    </button>
{:else if as === 'a'}
    <a class={className} {href} {disabled}>
        <slot/>
    </a>
{:else if as === 'summary'}
    <summary class={className} {disabled}>
        <slot/>
    </summary>
{:else if as === 'input'}
    <input class={className} {value} {disabled}/>
{/if}

<style lang="postcss">
    .button {
        display: flex;
        justify-content: flex-start;
        align-items: center;
        padding: 0.25rem 1rem;
        grid-gap: 0.4rem;
        cursor: pointer;
        position: relative;

        box-shadow: var(--shadow-inset-sm);

        background-color: var(--color-button-bg);
        border-radius: var(--rounded);
        transition: opacity 0.5s ease-in-out, filter 0.5s ease-in-out;

        &:hover {
            background-color: var(--color-button-bg-hover);
        }

        &--color {
            &-raised {
                background-color: var(--color-raised-bg);

                &:hover {
                    background-color: var(--color-raised-bg-hover);
                }
            }

            &-primary {
                background-color: var(--color-brand);
                color: var(--color-brand-contrast);

                &:hover {
                    background-color: var(--color-brand-dark);
                }
            }

            &-primary-light {
                background-color: var(--color-brand-light);
                transition: filter 0s ease-in-out;

                &:hover {
                    background-color: var(--color-brand-light);
                    filter: brightness(0.9);
                }
            }

            &-transparent {
                background-color: transparent;
                box-shadow: none;
            }

            &-danger {
                background-color: var(--color-badge-red-dot);
                color: var(--color-brand-contrast);

                &:hover {
                    background-color: var(--color-badge-red-text);
                }
            }

            &-danger-light {
                color: var(--color-danger-text);
                transition: filter 0s ease-in-out;

                &:hover {
                    filter: brightness(0.9);
                }
            }
        }

        &:disabled {
            opacity: 50%;
            cursor: not-allowed;
            filter: grayscale(50%);

            /* Not ideal, but preventing events being fired needs to be implemented */
            pointer-events: none;
        }

        &--pad-even {
            padding: 0.5rem;
            font-size: 1rem;
            line-height: 0;
            min-width: 2rem;
            min-height: 2rem;
            justify-content: center;
        }

        &.is-iconified {
            padding: 0.25rem 0.75rem;
        }

        &.has-badge::after {
            content: '';
            width: 0.5rem;
            height: 0.5rem;
            border-radius: var(--rounded-max);
            background-color: var(--color-brand);
            position: absolute;
            top: 0.5rem;
            right: 0.5rem;
        }
    }
</style>
