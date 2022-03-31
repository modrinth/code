<script lang="ts">
    import OmorphiaLogo from '../assets/omorphia.svg'
    import IconLogoGithub from 'virtual:icons/carbon/logo-github'
    import IconChat from 'virtual:icons/heroicons-outline/chat-alt-2'
    import { onMount } from 'svelte'

    let headerElement

    onMount(() => {
        let lastScrollTop: number
        window.addEventListener('scroll', () => {
            let scrollTop = window.pageYOffset || document.documentElement.scrollTop
            if (scrollTop > lastScrollTop && headerElement) {
                headerElement.style.top = 'calc(var(--header-height) * -1)'
            } else if (headerElement) {
                headerElement.style.top = '0'
            }
            lastScrollTop = scrollTop
        })
    })
</script>

<header class="header" bind:this={headerElement}>
    <OmorphiaLogo class="header__logo"/>
    <div class="header__title">Omorphia</div>
    <div class="header__links">
        <a class="hide-sm" href="https://modrinth.com">Modrinth.com</a>
        <span class="spacer-dot hide-sm"></span>
        <a href="https://www.npmjs.com/package/omorphia">NPM</a>
        <span class="spacer-dot"></span>
        <a href="https://rewrite.modrinth.com/discord">
            <IconChat/>
        </a>
        <a href="https://github.com/modrinth/omorphia">
            <IconLogoGithub/>
        </a>
    </div>
</header>


<style lang="postcss">
    .header {
        display: flex;
        grid-gap: 10px;
        align-items: center;
        flex-wrap: wrap;
        padding: 16px 24px;
        position: fixed;
        height: var(--header-height);
        left: 0;
        right: 0;
        top: 0;
        z-index: 10;
        background-color: hsl(0, 0%, 100%);
        box-shadow: hsla(221, 39%, 11%, 0.2) 0 2px 4px 0, hsla(221, 39%, 11%, 0.05) 0 -2px 2px 0 inset;
        transition: top 0.3s ease-in-out;

        @media (--sm) {
            padding: 10px 32px;
        }

        :global(&__logo) {
            max-width: 32px;
            min-width: 32px;
            aspect-ratio: 1 / 1;
        }

        &__title {
            font-size: 20px;
            font-weight: 600;
        }

        &__links {
            margin-left: auto;
            grid-gap: 16px;
            align-items: center;
            justify-content: center;
            display: flex;

            :global(svg) {
                height: 22px;
                width: auto;
            }

            .hide-sm {
                display: none;

                @media (--sm) {
                    display: flex;
                }
            }

            a {
                text-decoration: none;

                &:not(:hover) {
                    color: unset;
                }
            }
        }
    }

    .spacer-dot {
        background-color: hsla(0, 0%, 0%, 0.2);
        border-radius: 999px;
        width: 5px;
        aspect-ratio: 1 / 1;
    }
</style>