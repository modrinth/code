### Default option example

```svelte example raised
<script lang="ts">
	import { Select } from 'omorphia'

	let sortMethod = 'downloads'
</script>

<Select
	options={[
		{ value: '', label: 'Relevance' },
		{ value: 'downloads', label: 'Downloads' },
		{ value: 'follows', label: 'Followers' },
		{ value: 'newest', label: 'Recently created' },
		{ value: 'updated', label: 'Recently updated' },
	]}
	bind:value={sortMethod} />
```

### Icon example

```svelte example raised
<script lang="ts">
	import { Select } from 'omorphia'
	import IconSun from 'virtual:icons/heroicons-outline/sun'
</script>

<Select
	options={[
		{ value: '1', label: 'Light' },
		{ value: '2', label: 'Dark' },
		{ value: '3', label: 'OLED' },
	]}
	icon={IconSun} />
```
