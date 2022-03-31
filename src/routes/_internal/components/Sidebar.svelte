<script lang="ts">
    import IconMenu from 'virtual:icons/lucide/menu'

    const components = Object.keys(import.meta.glob('../../components/**'))
        .map(it => it.replace('../../components/', '').replace('.md', ''))
        .sort();

    const classes = Object.keys(import.meta.glob('../../classes/**'))
        .map(it => it.replace('../../classes/', '').replace('.md', ''))
        .sort();

    let slideIn = false
</script>

<nav class="sidebar" class:slideIn>
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

    <button class="sidebar__toggle" on:click={() => slideIn = !slideIn}><IconMenu /></button>
</nav>

<style lang="postcss">
    :root {
        --sidebar-color: hsl(220, 15%, 40%);
    }

    .sidebar {
        display: flex;
        flex-direction: column;
        grid-gap: 2rem;
        background-color: var(--sidebar-color);
        color: hsl(216, 10%, 80%);
        height: 100vh;
        max-height: 100vh;
        overflow-y: auto;
        width: var(--sidebar-width);
        max-width: 70vw;
        position: fixed;
        left: -100%;
        top: 0;
        z-index: 5;
        padding: 88px 32px 32px;
        transition: left 0.2s ease-in-out;
        box-shadow: 2px 0px 4px hsla(221, 39%, 11%, 0.2);

        @media (--md) {
            left: 0;
        }


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

        &__toggle {
            position: fixed;
            left: 16px;
            bottom: 16px;
            padding: 8px;
            aspect-ratio: 1 / 1;
            background-color: var(--accent-color);
            z-index: 20;
            border-radius: var(--rounded);
            color: white;
            box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
            transition: left 0.2s cubic-bezier(.38,.52,.37,1.27);

            :global(.icon) {
                width: 32px;
                height: auto;
            }

            @media (--md) {
                visibility: hidden;
            }
        }

        &.slideIn {
            left: 0;

            .sidebar__toggle {
                left: calc(32px + min(70vw, var(--sidebar-width)))
            }
        }
    }
</style>