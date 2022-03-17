<script lang="ts">
    export let href: string;
</script>

<a {href}><slot /></a>

<style lang="postcss">
    a {
        color: blue;
    }
</style>