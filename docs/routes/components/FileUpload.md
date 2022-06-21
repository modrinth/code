### Single constrained example

```svelte example raised column
<script lang="ts">
	import { Field, FileUpload } from 'omorphia'

	let file: File
</script>

<Field label="Upload image">
	<FileUpload accept="image/*" constrained bind:file />
</Field>

File name: {file?.name}
```

### Multiple example

```svelte example raised column
<script lang="ts">
	import { Field, FileUpload } from 'omorphia'

	let files: File[] = []
</script>

<Field label="Upload file">
	<FileUpload accept="*" multiple bind:files />
</Field>

Count: {files.length}
```
