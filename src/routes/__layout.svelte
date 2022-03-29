<script lang="ts">
    import '$lib/styles.postcss'
    import './_internal/styles/prism-one-dark.css'
    import './_internal/styles/gh-markdown.postcss'
    import Sidebar from './_internal/components/Sidebar.svelte'
    import Header from './_internal/components/Header.svelte'
</script>

<div class="app">
    <Header/>
    <Sidebar/>
    <main class="app__content">
        <slot/>
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
        display: grid;
        min-height: 100vh;
        overflow: hidden;
        --sidebar-width: 325px;
        --header-height: 56px;
        @media (width <= 500px) {
            display: flex;
            flex-direction: column;
        }

        &__content {
            padding: var(--header-height) 0 0 var(--sidebar-width);
            overflow-x: hidden;
            overflow-y: auto;
            background-color: hsl(220, 13%, 91%);

            :global {
                a:not(.example__preview *) {
                    color: var(--accent-color);

                    &:hover {
                        text-decoration: underline;
                    }
                }

                h1:not(.example__preview *) {
                    font-size: 54px;
                }

                p:not(.example__preview *) {
                    line-height: 1.5;
                }
            }
        }
    }
</style>