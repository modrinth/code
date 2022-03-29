<script lang="ts">
    const components = Object.keys(import.meta.glob('../../components/**'))
        .map(it => it.replace('../../components/', '').replace('.md', ''))
        .sort();

    const classes = Object.keys(import.meta.glob('../../classes/**'))
        .map(it => it.replace('../../classes/', '').replace('.md', ''))
        .sort();
</script>

<nav class="sidebar">
    <div class="section">
        <span class="section__title">Getting started</span>
        <a href="/" class="section__link">Introduction</a>
        <a href="/getting-started/icons" class="section__link">Using Icons</a>
        <a href="/getting-started/postcss" class="section__link">PostCSS config</a>
        <a href="/getting-started/css" class="section__link">Writing CSS</a>
    </div>

    <div class="section">
        <span class="section__title">Components</span>
        {#each components as component}
            <a href="/components/{component}" class="section__link">{component}</a>
        {/each}
    </div>

    <div class="section">
        <span class="section__title">Classes</span>
        {#each classes as page}
            <a href="/classes/{page}" class="section__link">{page}</a>
        {/each}
    </div>
</nav>

<style lang="postcss">
    .sidebar {
        display: flex;
        flex-direction: column;
        grid-gap: 2rem;
        background-color: hsl(220, 15%, 40%);
        color: hsl(216, 10%, 80%);
        padding: 5.5rem 2rem 2rem;
        height: 100vh;
        width: calc(var(--sidebar-width) - 3rem);
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
                color: hsl(216, 10%, 90%);
                text-decoration: none;

                &:hover {
                    color: white;
                    text-decoration: underline;
                }
            }
        }
    }
</style>