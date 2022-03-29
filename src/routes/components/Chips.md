### Simple example

```svelte example
<script lang="ts">
    import Chips from "omorphia/components/Chips.svelte";
</script>

<Chips options={[
    {
        label: 'One',
        value: 'one'
    },
    {
        label: 'Two',
        value: 'two'
    }]}
/>
```


### Force an option to be selected with `neverEmpty`

```svelte example
<script lang="ts">
    import Chips from "omorphia/components/Chips.svelte";

    let foo = 'modpack'
</script>

<Chips neverEmpty bind:value={foo} options={[
    {
        label: 'Mod',
        value: 'mod'
    },
    {
        label: 'Modpack',
        value: 'modpack'
    },
    {
        label: 'World',
        value: 'world'
    }]}
/>
```
