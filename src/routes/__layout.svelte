<script lang="ts">
    import "$lib/styles/variables.postcss";
    import "../lib/styles/themes/old.postcss";
    import Sidebar from "./_docs/Sidebar.svelte";
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
        grid-template-columns: 250px 1fr;
        @media(width <= 500px) {
            display: flex;
            flex-direction: column;
        }

        &__content {
            padding: 1rem max(10vw, 1rem);
        }
    }
</style>