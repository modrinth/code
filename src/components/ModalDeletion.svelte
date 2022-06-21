<script lang="ts">
	import Modal from './Modal.svelte'
	import Button from './Button.svelte'
	import TextInput from './TextInput.svelte'
	import { t } from 'svelte-intl-precompile'
	import IconExclamation from 'virtual:icons/heroicons-outline/exclamation'
	import IconTrash from 'virtual:icons/heroicons-outline/trash'
	import { markdown } from '../utils'
	import Field from './Field.svelte'
	import { createEventDispatcher } from 'svelte'

	export let key: string
	export let type: 'project' | 'version' | 'account' | 'image'

	export let open = false

	let data: { key?: string } = {}

	const dispatch = createEventDispatcher()
</script>

<!-- @ts-ignore -->
<Modal title={$t(`modal.deletion.${type}.title`)} bind:open bind:data>
	<svelte:fragment slot="trigger" let:trigger>
		<slot name="trigger" {trigger} />
	</svelte:fragment>

	{#if type === 'account' || 'project'}
		<div class="important-banner">
			<IconExclamation /><span>{$t('modal.deletion.generic.important')}</span>
		</div>
	{/if}
	{@html markdown($t(`modal.deletion.${type}.description`))}
	<Field label={$t('modal.deletion.generic.verify', { values: { key } })} let:id>
		<TextInput
			placeholder={$t('modal.deletion.generic.placeholder', { values: { key } })}
			bind:value={data.key}
			{id} />
	</Field>

	<svelte:fragment slot="button" let:close>
		<Button
			color="danger"
			disabled={key !== data.key}
			on:click={() => {
				close()
				dispatch('deletion')
			}}>
			<IconTrash />
			{$t(`modal.deletion.${type}.action`)}
		</Button>
	</svelte:fragment>
</Modal>

<style lang="postcss">
	.important-banner {
		margin: -1.5rem -1rem 0.5rem;
		background-color: var(--color-danger-bg);
		padding: 1rem 1.25rem;
		display: flex;
		align-items: center;
		gap: 1rem;
		color: var(--color-danger-text);
		border-color: var(--color-danger-text);
		border-width: 0.15rem 0;
		border-style: solid;

		:global(.icon) {
			height: 1.5rem;
			width: 1.5rem;
		}
	}
</style>
