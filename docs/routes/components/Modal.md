```svelte example raised
<script lang="ts">
	import { Modal, Button } from 'omorphia'
	import IconArrowRight from 'virtual:icons/heroicons-outline/arrow-right'
	import IconCheck from 'virtual:icons/heroicons-outline/check'
</script>

<Modal title="Example modal" danger let:trigger>
	<Button on:click={trigger} slot="trigger">Open modal</Button>

	<p>Secret message goes here!</p>
	<Button color="primary" slot="button"><IconArrowRight /> Continue</Button>
</Modal>

<Modal let:trigger size="sm">
	<Button on:click={trigger} slot="trigger">Confirm modal</Button>

	Are you sure you want to delete this gallery image?
	<Button color="primary" slot="button"><IconCheck /> Confirm</Button>
</Modal>
```
