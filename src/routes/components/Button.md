<script lang="ts">
    import Button from "$lib/components/Button.svelte";
    import Example from "../_internal/components/Example.svelte"
</script>

<Example code={`<Button>Eat cake</Button>
<Button size="sm" color="primary">Small piece</Button>
<Button size="lg" color="danger">Big part</Button>
<Button disabled>Nice try</Button>`}>
<Button>Eat cake</Button>
<Button size="sm" color="primary">Small piece</Button>
<Button size="lg" color="danger">Big part</Button>
<Button disabled>Nice try</Button>
</Example>