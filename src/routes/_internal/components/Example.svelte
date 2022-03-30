<script lang="ts">
    import Button from '$lib/components/Button.svelte'
    import IconMoon from 'virtual:icons/heroicons-outline/moon'
    import IconSun from 'virtual:icons/heroicons-outline/sun'

    let theme = 'dark'
    let background: 'var(--color-raised-bg)' | 'var(--color-bg)' = 'var(--color-bg)'
</script>

<div class="example">
    <div class="example__preview theme-{theme} base" style:background={background}>
        <div class="example__preview__options">
            <Button color="primary-light" on:click={() => theme === 'light' ? theme = 'dark' : theme = 'light'}>
                {#if theme === 'light'}
                    <IconMoon/>
                {:else}
                    <IconSun/>
                {/if}
            </Button>
        </div>
        <slot name="example"/>
    </div>
    <pre class="example__code language-svelte"><slot name="code"/></pre>
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

            &__options {
                position: absolute;
                top: 0;
                right: 0;
                padding: 8px;
                display: flex;
                justify-content: flex-end;
                z-index: 100;
            }
        }

        &__code {
            margin: 0;
            border-radius: var(--rounded-sm-bottom) !important;
            background: hsl(220, 13%, 22%);
        }
    }
</style>
