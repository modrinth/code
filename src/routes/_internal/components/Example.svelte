<script lang="ts">
    import { Button } from 'omorphia/'
    import IconMoon from 'virtual:icons/heroicons-outline/moon'
    import IconSun from 'virtual:icons/heroicons-outline/sun'

    let theme = 'dark'
    let background: 'var(--color-raised-bg)' | 'var(--color-bg)' = 'var(--color-bg)'
</script>

<div class="example">
    <div class="example__preview theme-{theme} base" style:background={background}>
        <slot name="example"/>
    </div>
    <div class="example__source">
        <div class="example__source__options">
            <Button color="primary-light" on:click={() => theme === 'light' ? theme = 'dark' : theme = 'light'}>
                {#if theme === 'light'}
                    <IconMoon/>
                {:else}
                    <IconSun/>
                {/if}
            </Button>
        </div>
        <pre class="example__source__code language-svelte"><slot name="code"/></pre>
    </div>
</div>

<style lang="postcss">
    .example {
        margin-bottom: 32px;

        &__preview {
            border-radius: var(--rounded-sm-top);
            border: solid 2px hsl(0, 0%, 20%);
            border-bottom: none;
            display: flex;
            grid-gap: 16px;
            flex-wrap: wrap;
            position: relative;
            justify-content: flex-start;
            z-index: 1;
        }

        &__source {
            position: relative;

            &__options {
                position: absolute;
                top: 0;
                right: 0;
                padding: 8px;
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
