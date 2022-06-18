<script lang="ts">
	// TODO: sizes
	// TODO: icon only buttons should have uniform padding
	import { createEventDispatcher } from 'svelte'
	import { classCombine } from '../utils/classCombine'

	/** The element to be styled as a button */
	export let as: 'button' | 'a' | 'summary' | 'input' = 'button'
	export let href = ''
	if (href) as = 'a'

	/** Use `value` if the button is an `<input`> */
	export let value = ''

	export let size: 'sm' | 'md' | 'lg' = 'md'
	export let color:
		| ''
		| 'raised'
		| 'primary'
		| 'primary-light'
		| 'secondary'
		| 'tertiary'
		| 'danger'
		| 'danger-light'
		| 'transparent' = ''

	/** Show notification badge in the upper right of button */
	export let badge = false

	export let disabled = false

	/**  Hover title for accessibility */
	export let title = ''

	/**  Link target */
	export let target = ''

	let className: string
	$: className = classCombine([
		'button',
		`button--size-${size}`,
		`button--color-${color}`,
		badge && 'has-badge',
	])

	const dispatch = createEventDispatcher()

	function dispatchClick() {
		if (!disabled) dispatch('click')
	}
</script>

{#if as === 'a'}
	<a class={className} {href} {disabled} {title} {target} on:click={dispatchClick}>
		<slot />
	</a>
{:else if as === 'input'}
	<input class={className} {value} {disabled} {title} on:click={dispatchClick} />
{:else}
	<svelte:element this={as} class={className} {disabled} {title} on:click={dispatchClick}>
		<slot />
	</svelte:element>
{/if}

<style lang="postcss">
	.button {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.5rem 1rem;
		min-width: 2rem;
		gap: 0.5rem;
		cursor: pointer;
		position: relative;
		line-height: 100%;

		color: var(--color-bg-contrast);

		box-shadow: var(--shadow-inset-sm);
		background-color: var(--color-button-bg);

		border-radius: var(--rounded);
		transition: opacity 0.5s ease-in-out, filter 0.2s ease-in-out, transform 0.05s ease-in-out,
			outline 0.2s ease-in-out;

		&:hover:not(&--color-transparent, &:disabled) {
			filter: brightness(0.85);
		}

		&:active:not(&--color-transparent, &:disabled) {
			transform: scale(0.95);
			filter: brightness(0.8);
		}

		&--color {
			&-raised {
				background-color: var(--color-raised-bg);
				box-shadow: var(--shadow-inset-sm), var(--shadow-raised);
			}

			&-primary {
				background-color: var(--color-brand);
				color: var(--color-brand-contrast);
			}

			&-primary-light {
				background-color: var(--color-brand-light);
			}

			&-secondary {
				background-color: var(--color-secondary);
				color: var(--color-brand-contrast);
			}

			&-tertiary {
				background-color: var(--color-tertiary);
			}

			&-transparent {
				background-color: transparent;
				box-shadow: none;
				filter: brightness(1) !important;

				&:hover {
					background-image: linear-gradient(rgba(0, 0, 0, 0.1) 0 0);
				}

				&:active {
					background-image: linear-gradient(rgba(0, 0, 0, 0.2) 0 0);
				}
			}

			&-danger {
				background-color: var(--color-badge-red-dot);
				color: var(--color-brand-contrast);
			}

			&-danger-light {
				background-color: var(--color-danger-bg);
				color: var(--color-danger-text);
			}
		}

		&:disabled {
			opacity: 50%;
			cursor: not-allowed;
			filter: grayscale(50%);
		}

		&--pad-even {
			padding: 0.5rem;
			font-size: 1rem;
			line-height: 0;
			min-width: 2rem;
			min-height: 2rem;
			justify-content: center;
		}

		&.has-badge::after {
			content: '';
			width: 0.5rem;
			height: 0.5rem;
			border-radius: var(--rounded-max);
			background-color: var(--color-brand);
			position: absolute;
			top: 0.5rem;
			right: 0.5rem;
		}
	}
</style>
