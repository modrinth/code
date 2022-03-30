```svelte example
<script lang="ts">
    import CheckboxList from "omorphia/components/CheckboxList.svelte";
    import IconSquare from 'virtual:icons/lucide/square'
    import IconCircle from 'virtual:icons/lucide/circle'
    import IconTriangle from 'virtual:icons/lucide/triangle'
    
    let selected = []
</script>

<CheckboxList 
    bind:value={selected}
    options={[
        {
            label: 'Circle',
            icon: IconCircle,
            value: 'CIR',
        },
        {
            label: 'Triangle',
            icon: IconTriangle,
            value: 'TRI',
        },
        {
            label: 'Square',
            icon: IconSquare,
            value: 'SQU',
        },
    ]} 
/>

Selected: {selected}
```
