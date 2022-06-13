```svelte example raised
<script lang="ts">
	import { Modal, Checkbox, Button } from 'omorphia'
	import IconArrowRight from 'virtual:icons/heroicons-outline/arrow-right'

	let open = false
</script>

<Checkbox bind:checked={open}>Open modal</Checkbox>

<Modal title="Example modal" danger cancelButton bind:open>
	<p>Secret message goes here!</p>
	<Button color="primary" slot="button"><IconArrowRight /> Continue</Button>
</Modal>
```
