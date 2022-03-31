<script lang="ts">
    import '$lib/styles.postcss'
    import './_internal/styles/prism-one-dark.css'
    import './_internal/styles/gh-markdown.postcss'
    import Sidebar from './_internal/components/Sidebar.svelte'
    import Header from './_internal/components/Header.svelte'
</script>

<div class="app theme-light">
    <Header/>
    <Sidebar/>
    <main class="app__content">
        <article>
            <slot />
        </article>
    </main>
</div>

<style lang="postcss">
    :global {
        html {
            overflow-y: scroll;
        }

        body {
            margin: 0;
            font-size: var(--body-font-size);
            font-family: var(--body-font);

            --accent-color: hsl(331, 80%, 45%);
            --accent-color-transparent: hsla(331, 80%, 45%, 0.15);
        }
    }

    .app {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        overflow: hidden;
        background-color: var(--color-bg);

        --sidebar-width: 275px;

        &__content {
            @media (--md) {
                padding-left: var(--sidebar-width);
            }
            overflow-x: hidden;
            overflow-y: auto;

            article {
                max-width: 800px;
                padding-block: 140px 64px;
                padding-inline: 24px;

                @media (--sm) {
                    padding-inline: 48px;
                    padding-block: 140px 24px;
                }
            }
        }
    }
</style>