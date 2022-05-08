### Single example

```svelte example
<script lang="ts">
    import { Button } from "omorphia";
    import IconDownload from 'virtual:icons/heroicons-outline/download'
</script>

<Button color="raised"><IconDownload /> Download</Button>
```

### Color variants example

```svelte example raised
<script lang="ts">
    import { Button } from "omorphia";
    import IconDownload from 'virtual:icons/heroicons-outline/download'
</script>

<div class="button-group">
    <Button>Default</Button>
    <Button color="raised">Raised</Button>
    <Button color="primary">Primary</Button>
    <Button color="primary-light">Light primary</Button>
    <Button color="secondary">Secondary</Button>
    <Button color="tertiary">Tertiary</Button>
    <Button color="danger">Danger</Button>
    <Button color="danger-light">Light danger</Button>
    <Button color="transparent">Transparent</Button>
    <Button disabled>Disabled</Button>
</div>
```

### With icons example

```svelte example raised
<script lang="ts">
    import { Button } from "omorphia";
    import IconDownload from 'virtual:icons/heroicons-outline/download'
    import IconHeart from 'virtual:icons/heroicons-outline/heart'
</script>

<div class="button-group">
    <Button color="primary"><IconDownload /></Button>
    <Button><IconHeart /> Follow mod </Button>
</div>
```
