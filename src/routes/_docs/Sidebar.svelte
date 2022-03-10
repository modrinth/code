<nav class="sidebar">
    <div class="section">
        <span class="section__title">Introduction</span>
        <a href="/" class="section__link">Welcome</a>
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

        .section {
            display: flex;
            flex-direction: column;
            grid-gap: 0.2rem;

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