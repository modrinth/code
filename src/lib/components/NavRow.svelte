<script lang="ts">
    import { page } from '$app/stores';

    interface Link {
        href: string;
        label: string;
    }

    export let links: Link[];
    export let query = '';

    export let resetScroll = false

    /** Path level in URL, zero-indexed */
    export let level = 0;

    let path: string[];
    $: path = [
        ...$page.url.pathname
            .substring(1)
            .split('/')
            .map((route) => '/' + route),
        '/',
    ];

    $: basePath = path.slice(0, level).join('');
</script>

<nav class="navigation">
    {#each links as link}
        <a
                href={query
				? link.href
					? `?${query}=${link.href}`
					: '?'
				: level === 0
				? link.href
				: basePath + link.href}

                on:click={() => { if (resetScroll) document.body.scrollTo(0,0)}}

                class="navigation__link"
                class:is-active={query
				? ($page.url.searchParams.get(query) || '') === link.href
				: path[level] === link.href || path[level] === link.href.slice(0, -1)}
                sveltekit:noscroll={!resetScroll}
        >{link.label}</a
        >
    {/each}
</nav>

<style lang="postcss">
    .navigation {
        display: flex;
        flex-direction: row;
        align-items: center;
        grid-gap: 1rem;
        flex-wrap: wrap;

        &__link {
            font-weight: var(--font-weight-bold);
            color: var(--color-text-light);

            &.is-active {
                color: var(--color-text);
                position: relative;

                &::after {
                    content: '';
                    display: block;
                    position: absolute;
                    bottom: -0.1rem;
                    width: 100%;
                    border-radius: var(--rounded-max);
                    height: 0.2rem;
                    background-color: var(--color-brand);
                }
            }
        }
    }
</style>
