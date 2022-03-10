<nav class="sidebar">
    <div class="section">
        <span class="section__title">Getting started</span>
        <a href="/" class="section__link">Introduction</a>
        <a href="/general/icons" class="section__link">Using Icons</a>
        <a href="/general/css" class="section__link">CSS configuration</a>
    </div>

    <div class="section">
        <span class="section__title">Components</span>
        {#each ['buttons'] as component}
            <a href="/components/{component}" class="section__link">{component}</a>
        {/each}
    </div>
</nav>

<style lang="postcss">
    .sidebar {
        display: flex;
        flex-direction: column;
        grid-gap: 2rem;
        background-color: hsl(216, 10%, 30%);
        color: hsl(216, 10%, 80%);
        padding: 1.5rem;
        height: 100vh;
        width: calc(var(--sidebar-size) - 3rem);
        position: fixed;
        left: 0;
        top: 0;

        .section {
            display: flex;
            flex-direction: column;
            grid-gap: 0.5rem;

            &__title {
                text-transform: uppercase;
                font-size: 12px;
            }

            &__link {
                text-transform: capitalize;
                color: hsl(216, 10%, 90%);
                text-decoration: none;
            }
        }
    }
</style>