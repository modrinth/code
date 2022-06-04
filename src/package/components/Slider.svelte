<script lang="ts">
	export let value: number
	export let min: number
	export let max: number
	export let id: string = undefined
</script>

<div class="slider">
	<input class="slider-input" type="range" name={id} {min} {max} bind:value />
	<span>{value}</span>
</div>

<style lang="postcss">
	.slider {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 8px;

		.slider-input {
			-webkit-appearance: none;
			appearance: none;
			border-radius: var(--rounded-sm);
			box-shadow: var(--shadow-inset-sm);
			background-color: var(--color-button-bg);
			border: none;
			padding: 0.25rem 0;
			width: 20rem;
			height: 0.5rem;
			max-width: 100%;
			cursor: pointer;

			&::-webkit-slider-thumb {
				cursor: ew-resize;
				-webkit-appearance: none;
				appearance: none;
				width: 0.5rem;
				height: 1.25rem;
				border-radius: var(--rounded-sm);
				background-color: var(--color-brand);

				&:hover {
					background-color: var(--color-brand-dark);
				}
			}
		}
	}
</style>
