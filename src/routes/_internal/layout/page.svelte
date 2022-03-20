<script lang="ts">
    import IconPencil from 'virtual:icons/heroicons-outline/pencil'
    import { page } from '$app/stores'
    import { onMount } from 'svelte'

    export let componentName = $page.url.pathname.includes('components') ? $page.url.pathname.replace('/components/', '') : undefined

    export let title = ''
    if (!title && componentName) title = componentName

    let pageUrl = `https://github.com/modrinth/omorphia/edit/main/src/routes/${$page.url.pathname.replace('/', '') || 'index'}.md`

    let api

    onMount(async () => {
        if (componentName) {
            api = (await import(`../../../lib/components/${componentName}.svelte?raw&api`)).default
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

    {#if componentName && api}
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
                    </tr>
                    </thead>
                    <tbody>
                    {#each api.data as prop}
                        <tr>
                            <td>{prop.name}</td>
                            <td>{prop.type.type}</td>
                            <td>{prop.defaultValue ?? ''}</td>
                            <td>{prop.readonly ? '[Read only] ' : ''}{prop.description?.replace('null', '') || ''}</td>
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

            <h2>Import</h2>
            <pre class="language-js"><code class="language-js"><span class="token keyword">import</span> <span class="token punctuation">&#123;</span> {componentName} <span class="token punctuation">}</span> <span class="token keyword">from</span> <span class="token string">"omorphia"</span></code></pre>
        </div>
    {/if}
</article>

<style lang="postcss">
    article {
        max-width: 800px;
        padding: 5rem 1rem;
    }

    .edit-link {
        display: flex;
        align-items: center;
        grid-gap: 8px;
        margin-bottom: 54px;
        color: var(--accent-color);
    }

    .extra-info {
        margin-top: 64px;
    }

    .api-table {
        border-collapse: collapse;
        margin-top: -6px;
    }

    .api-table tr {
        background-color: transparent;
        border: none;
    }

    .api-table tbody {
        border: 2px solid grey;
    }

    .api-table th {
        text-transform: uppercase;
        font-size: 12.5px;
        border: none;
    }
</style>