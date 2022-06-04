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
		{ value: 'newest', label: 'Recently' },
		{ value: 'updated', label: 'Recently updated really fast whoot' },
	]}
	bind:value={sortMethod} />
```
