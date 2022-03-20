<script lang="ts">
    import Pagination from "$lib/components/Pagination.svelte"
    import Example from "../_internal/components/Example.svelte"
</script>

Use pagination to show a set of page numbers and navigation directions to move through paginated data.

<Example code={`<Pagination page={20} count={50} />`} background="transparent">
    <Pagination page={20} count={50} />
</Example>
