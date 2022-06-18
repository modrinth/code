<script lang="ts">
	export let label = ''
	/** Supports `green`, `yellow`, `red`, & `gray` */
	export let color = 'gray'
</script>

<div class="badge badge--color-{color}">
	{label}
</div>

<style lang="postcss">
	.badge {
		font-weight: var(--font-weight-bold);
		display: inline;
		position: relative;
		padding-left: 1rem;
		line-height: 1em;

		&--color-green {
			color: var(--color-badge-green-text);
			--color-dot: var(--color-badge-green-dot);
		}

		&--color-yellow {
			color: var(--color-badge-yellow-text);
			--color-dot: var(--color-badge-yellow-dot);
		}

		&--color-red {
			color: var(--color-badge-red-text);
			--color-dot: var(--color-badge-red-dot);
		}

		&--color-gray {
			color: var(--color-badge-gray-text);
			--color-dot: var(--color-badge-gray-dot);
		}

		&::before {
			content: '';
			display: inline-block;
			width: 0.5rem;
			aspect-ratio: 1 / 1;
			border-radius: 50%;
			background-color: var(--color-dot);
			position: absolute;
			left: 0;
			bottom: 25%;
		}
	}
</style>
