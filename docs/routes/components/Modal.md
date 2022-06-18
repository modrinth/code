```svelte example raised
<script lang="ts">
	import { Modal, Button } from 'omorphia'
	import IconArrowRight from 'virtual:icons/heroicons-outline/arrow-right'
</script>

<Modal title="Example modal" danger cancelButton let:trigger>
	<Button on:click={trigger} slot="trigger">Open modal</Button>

	<p>Secret message goes here!</p>
	<Button color="primary" slot="button"><IconArrowRight /> Continue</Button>
</Modal>
```
