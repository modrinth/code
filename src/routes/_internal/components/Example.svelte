<script lang="ts">
    import Prism from 'prismjs';
    import 'prism-svelte';

    export let background: 'var(--color-raised-bg)' | 'transparent' = 'var(--color-raised-bg)'

    export let code = ''
    const highlighted = Prism.highlight(code.trim(), Prism.languages.svelte, 'svelte');
</script>

<div class="example theme-light">
    <div class="example__preview" style:background={background}>
        <slot />
    </div>
    <pre class="example__code language-">{@html highlighted}</pre>
</div>

<style lang="postcss">
    .example {
        &__preview {
            padding: 16px;
            border-radius: var(--rounded-sm-top);
            border: solid 2px hsl(0, 0%, 20%);
            border-bottom: none;
            display: flex;
            grid-gap: 16px;
            flex-wrap: wrap;
        }

        &__code {
            margin: 0;
            border-radius: var(--rounded-sm-bottom) !important;
        }
    }
</style>
