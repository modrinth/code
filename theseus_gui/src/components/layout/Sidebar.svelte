<script lang="ts">
    import VerticalNav from '../components/VerticalNav.svelte'
    import IconHome from 'virtual:icons/lucide/home'
    import IconSearch from 'virtual:icons/heroicons-outline/search'
    import IconLibrary from 'virtual:icons/lucide/library';
    import IconSettings from 'virtual:icons/lucide/settings'
    import { page } from "$app/stores";
</script>

<div class="sidebar">
    <div class="account">
        <div class="account__heads">
            <img src="https://mc-heads.net/avatar/venashial" alt="Minecraft head"/>
        </div>

        <a class="account__info" href="/settings/accounts">
            <div>venashial</div>
            <div class="account__info__manage">Manage accounts</div>
        </a>
    </div>

    <VerticalNav items={[
        {
            label: 'Home',
            href: '',
            icon: IconHome,
        },
        {
            label: 'Search',
            href: 'search',
            icon: IconSearch,
        },
        {
            label: 'Library',
            href: 'library',
            icon: IconLibrary,
        }
    ]}/>

    <a class="nav-item" href="/settings" class:active={$page.url.pathname.startsWith('/settings')}>
        <IconSettings />
        Settings
    </a>
</div>

<style lang="postcss">
    .sidebar {
        display: flex;
        flex-direction: column;
        padding: 1rem;
        grid-gap: 2rem;
        height: 100%;
        background-color: var(--sidebar-bg);

        .account {
            display: flex;
            grid-gap: 0.75rem;

            &__heads {
                img {
                    width: 2.5rem;
                    border-radius: var(--rounded-sm);
                }
            }

            &__info {
                &__manage {
                    color: var(--color-text-light)
                }
            }
        }

        *:last-child {
            margin-top: auto;
        }
    }
</style>
