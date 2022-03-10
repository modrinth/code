<script lang="ts">
    import "$lib/styles/variables.postcss"
    import "$lib/styles/themes.postcss"
    import "./_docs/prism-one-dark.css"
    import Sidebar from "./_docs/Sidebar.svelte"
</script>

<div class="app">
    <Sidebar />
    <main class="app__content">
        <slot />
    </main>
</div>

<style lang="postcss">
    :global(body) {
        margin: 0;
        font-size: var(--body-font-size);
        font-family: var(--body-font);
    }

    .app {
        display: grid;
        min-height: 100vh;
        --sidebar-size: 250px;
        padding-left: var(--sidebar-size);
        @media (width <= 500px) {
            display: flex;
            flex-direction: column;
        }
        overflow: hidden;

        &__content {
            padding: 1rem max(8vw, 1rem);
            max-width: 100%;
            overflow-x: hidden;
            overflow-y: auto;

            :global(a) {
                color: hsl(216, 50%, 50%);
            }
        }
    }
</style>