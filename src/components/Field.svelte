<script lang="ts">
	import { uniqueId } from '../utils/uniqueId'

	export let required = false
	export let label: string
	export let helper = ''

	const id = `field-${uniqueId()}`
</script>

<div class="field">
	<label for={id} class="field__label" class:required>
		<span class="field__label__title">{@html label}</span>
		{#if helper}
			<span class="field__label__helper">{helper}</span>
		{/if}
	</label>
	<slot {id} />
</div>

<style lang="postcss">
	.field {
		display: flex;
		flex-direction: column;
		grid-gap: 0.5rem;
		min-width: 0;

		&__label {
			display: flex;
			flex-direction: column;
			grid-gap: 0;

			&__title {
				font-weight: var(--font-weight-bold);

				:global(i) {
					font-weight: var(--font-weight-regular);
				}
			}

			&__helper {
				font-size: var(--font-size-sm);
				color: var(--color-text-light);
			}
		}
	}
</style>
