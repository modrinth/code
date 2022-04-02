<script lang="ts">
    import IconChevronDown from 'virtual:icons/lucide/chevron-down'
    import { debounce } from 'throttle-debounce'
    import { clickOutside } from 'svelte-use-click-outside'
    import { onMount } from 'svelte'

    interface Option {
        label: string;
        value: string | number;
    }

    export let options: Option[] = []
    export let value: string | number
    export let selected: Option = options.find((option) => option.value === (value || ''))
    export let color = ''
    export let label = ''
    export let icon = null

    let open = false

    $: if (selected) {
        value = selected.value
    }

    const minWidth = options
			.map(it => it.label || it.value)
			.reduce((it, acc) => String(it).length > acc ? it : acc, '')
			.length * 9

    let shouldOpenUp = false
    let element: HTMLElement

    const checkShouldOpenUp = debounce(100, false, () => {
        if (element) {
            const bounding = element.getBoundingClientRect()

            shouldOpenUp =
                bounding.bottom + 32 * options.length + 16 >
                (window.innerHeight || document.documentElement.clientHeight)
        }
    })

    onMount(() => {
        checkShouldOpenUp()
        window.addEventListener('resize', checkShouldOpenUp)
    })

    function keydown(event: KeyboardEvent) {
        if ((event.key === ' ' || event.key === 'Enter') && !open) {
            open = true
        } else if (event.key === 'ArrowUp') {
            if (selected) {
                const index = options.findIndex((option) => option.value === selected.value)
                if (index > 0) {
                    selected = options[index - 1]
                }
            }
        } else if (event.key === 'ArrowDown') {
            if (selected) {
                const index = options.findIndex((option) => option.value === selected.value)
                if (index < options.length - 1) {
                    selected = options[index + 1]
                }
            }
        } else if ((event.key === 'Escape' || event.key === 'Enter') && open) {
            open = false
        }
    }
</script>

<div
        class="select select--color-{color}"
        class:is-open={open}
        class:select--opens-up={shouldOpenUp}
        use:clickOutside={() => {
		open = false;
	}}
        bind:this={element}
        tabindex="0"
        on:keydown={keydown}
>
    <div
            class="select__input"
            on:click={() => {
			open = !open;
		}}
    >
        {#if icon}
            <svelte:component this={icon}/>
        {/if}
        <span class="select__input__value" style:min-width="{minWidth}px">{label || selected?.label || value || 'Choose...'}</span>
        {#if !icon}
            <div class="select__input__arrow">
                <slot name="expandIcon">
                    <IconChevronDown/>
                </slot>
            </div>
        {/if}
    </div>
    {#if open}
        <ul class="select__options">
            {#each options as option (option.value)}
                <li
                        on:click={() => {
						selected = option;
						open = false;
					}}
                        class:is-selected={selected?.value === option.value}
                >
                    {option.label || option.value}
                </li>
            {/each}
        </ul>
    {/if}
</div>

<style lang="postcss">
    .select {
        position: relative;
        display: flex;
        flex-direction: column;
        color: var(--color-button-text);
        cursor: pointer;
        border-radius: var(--rounded);
        align-self: flex-start;

        &__input {
            display: flex;
            width: 100%;
            justify-content: space-between;
            align-items: center;
            padding: 0.25rem 0.9rem;
            grid-gap: 0.4rem;
            background-color: var(--color-button-bg);
            box-shadow: var(--shadow-inset-sm);
            border-radius: var(--rounded);
        }

        &__options {
            list-style-type: none;
            display: flex;
            flex-direction: column;
            width: 100%;
            padding: 0;
            margin: 0;
            position: absolute;
            top: 100%;
            background-color: var(--color-button-bg);
            border-radius: var(--rounded-bottom);
            box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
            overflow: hidden;
            border-top: 0.1rem solid var(--color-divider);
            z-index: 5;

            li {
                padding: 0.25rem 1rem;

                &:hover {
                    background-color: var(--color-button-bg-hover);
                }

                &.is-selected {
                    background-color: var(--color-brand-dark);
                    color: var(--color-brand-contrast);
                    cursor: default;
                }
            }
        }

        &.is-open {
            z-index: 10;

            .select__input {
                border-radius: var(--rounded-top);
                box-shadow: none;

                &__arrow {
                    transform: rotate(180deg);
                }
            }
        }

        &--color-raised {
            > * {
                background-color: var(--color-raised-bg);
            }
        }

        &--opens-up {
            .select__options {
                bottom: 100%;
                top: auto;
                border-radius: var(--rounded-top);
                box-shadow: none;
                border-top: none;
                border-bottom: 0.1rem solid var(--color-divider);
            }

            &.is-open .select__input {
                border-radius: var(--rounded-bottom);
            }
        }
    }
</style>
