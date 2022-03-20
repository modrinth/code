<script lang="ts">
    import Chips from "$lib/components/Chips.svelte";
    import Example from "../_internal/components/Example.svelte"

    let foo = 'modpack'
</script>

### Simple example

<Example code={`
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
`}>
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
</Example>


### Force an option to be selected with `neverEmpty`

<Example code={`
<script>
    let foo = 'modpack';
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
`}>
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
</Example>