<script lang="ts">
    import IconPencil from 'virtual:icons/heroicons-outline/pencil'
    import { page } from '$app/stores'

    export let title
    let pageUrl = `https://github.com/modrinth/omorphia/edit/main/src/routes/${$page.url.pathname.replace('/', '') || 'index'}.md`
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
        margin-bottom: 64px;
    }
</style>