```svelte example raised
<script lang="ts">
	import { ModalDeletion, Button } from 'omorphia'

	let open = false
</script>

<Button
	color="danger"
	on:click={() => {
		open = true
	}}>
	Delete account
</Button>
<ModalDeletion type="account" key="venashial" bind:open />
```
