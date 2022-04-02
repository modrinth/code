### Single example

```svelte example
<script lang="ts">
    import { Button } from "omorphia";
    import IconDownload from 'virtual:icons/heroicons-outline/download'
</script>

<Button color="primary"><IconDownload /> Download</Button>
```

### Group example

```svelte example
<script lang="ts">
    import { Button } from "omorphia";
    import IconDownload from 'virtual:icons/heroicons-outline/download'
</script>

<div class="button-group">
    <Button>Default button</Button>
    <Button color="raised">Raised button</Button>
    <Button color="primary">Primary button</Button>
    <Button color="primary-light">Light primary button</Button>
    <Button color="danger">Danger button</Button>
    <Button color="danger-light">Light danger button</Button>
    <Button color="transparent">Transparent button</Button>
    <Button disabled>Disabled button</Button>
    <Button color="primary"><IconDownload /></Button>
</div>
```