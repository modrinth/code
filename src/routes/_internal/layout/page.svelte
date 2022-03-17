<script lang="ts">
    import IconPencil from 'virtual:icons/heroicons-outline/pencil'
    import { page } from '$app/stores'
    import { onMount } from 'svelte'

    export let title
    export let component
    let pageUrl = `https://github.com/modrinth/omorphia/edit/main/src/routes/${$page.url.pathname.replace('/', '') || 'index'}.md`

    let api

    onMount(async () => {
        if (component) {
            api = (await import(`../../../lib/components/${component}.svelte?raw&api`)).default
        }
    })
</script>

<svelte:head>
    <title>{title ? `${title} - Omorphia` : 'Omorphia'}</title>
</svelte:head>

<article>
    {#if title}<h1>{title}</h1>{/if}
    <a class="edit-link" href={pageUrl}>
        <IconPencil/>
        Edit this page on GitHub</a>
    <slot/>

    {#if component && api}
        <div class="extra-info">
            {#if api.data.length > 0}
                <h2>Properties</h2>
                <table class="api-table">
                    <thead>
                    <tr>
                        <th>Name</th>
                        <th>Type</th>
                        <th>Default</th>
                        <th>Description</th>
                        <th>Read only</th>
                    </tr>
                    </thead>
                    <tbody>
                    {#each api.data as prop}
                        <tr>
                            <td>{prop.name}</td>
                            <td>{prop.type.type}</td>
                            <td>{prop.defaultValue}</td>
                            <td>{prop.description?.replace('null', '') || ''}</td>
                            <td>{prop.readonly}</td>
                        </tr>
                    {/each}
                    </tbody>
                </table>
            {/if}
            {#if api.events.length > 0}
                <h2>Events</h2>
                <table class="api-table">
                    <thead>
                    <tr>
                        <th>Name</th>
                        <th>Forwarded</th>
                        <th>Description</th>
                    </tr>
                    </thead>
                    <tbody>
                    {#each api.events as event}
                        <tr>
                            <td>{event.name}</td>
                            <td>{!!event.parent}</td>
                            <td>{event.description?.replace('null', '') || ''}</td>
                        </tr>
                    {/each}
                    </tbody>
                </table>
            {/if}
            {#if api.slots.length > 0}
                <h2>Slots</h2>
                <table class="api-table">
                    <thead>
                    <tr>
                        <th>Name</th>
                        <th>Description</th>
                    </tr>
                    </thead>
                    <tbody>
                    {#each api.slots as slot}
                        <tr>
                            <td>{slot.name}</td>
                            <td>{slot.description?.replace('null', '') || ''}</td>
                        </tr>
                    {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {/if}
</article>

<style lang="postcss">
    article {
        max-width: 800px;
        padding: 5rem max(8vw, 1rem);
    }

    .edit-link {
        display: flex;
        align-items: center;
        grid-gap: 8px;
        margin-bottom: 54px;
    }

    .extra-info {
        margin-top: 48px;
    }

    .api-table {
        border-collapse: collapse;

        tr {
            background-color: transparent;
            border: none;
        }

        tbody {
            border: 2px solid grey;
        }

        th {
            text-transform: uppercase;
            font-size: 12.5px;
            border: none;
        }
    }
</style>