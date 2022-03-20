<script lang="ts">
    export let href: string;
</script>

<a {href}><slot /></a>

<style lang="postcss">
    a {
        color: var(--color-link);

        &:hover {
            text-decoration: underline;
        }
    }
</style>