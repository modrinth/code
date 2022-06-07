<script lang="ts">
	import IconChevronDown from 'virtual:icons/lucide/chevron-down'
	import IconCheck from 'virtual:icons/heroicons-outline/check'
	import { debounce } from 'throttle-debounce'
	import { clickOutside } from 'svelte-use-click-outside'
	import { onMount } from 'svelte'

	interface Option {
		label: string
		value: string | number
	}

	export let options: Option[] = []
	export let value: string | number
	export let selected: Option
	export let color = ''
	export let label = ''
	export let icon = null

	let open = false

	$: if (options) {
		setSelected()
	}

	function setSelected() {
		selected = options.find((option) => option.value === (value || ''))
	}

	$: if (selected) {
		value = selected.value
	}

	// Returns the width of a string based on the font size and family
	const getTextWidth = Object.assign(
		(text: string, font: string) => {
			if (typeof document !== 'undefined') {
				const canvas =
					getTextWidth.canvas || (getTextWidth.canvas = document.createElement('canvas'))
				const context = canvas.getContext('2d')
				context.font = font
				const metrics = context.measureText(text)
				return metrics.width
			} else {
				// Return estimate if SSR
				return text.length * 7.75
			}
		},
		// Reuses the same canvas object
		{ canvas: null }
	)

	const minWidth = Math.max(
		...options.map((it) => getTextWidth(String(it.label || it.value), '16px Inter'))
	)

	let shouldOpenUp = false
	let element: HTMLElement

	const checkShouldOpenUp = debounce(100, false, () => {
		if (element) {
			const bounding = element.getBoundingClientRect()

			shouldOpenUp =
				bounding.bottom + 32 * options.length + 16 >
				(window.innerHeight || document.documentElement.clientHeight)
		}
	})

	onMount(() => {
		checkShouldOpenUp()
		window.addEventListener('resize', checkShouldOpenUp)
	})

	function keydown(event: KeyboardEvent) {
		if ((event.key === ' ' || event.key === 'Enter') && !open) {
			open = true
		} else if (event.key === 'ArrowUp') {
			if (selected) {
				const index = options.findIndex((option) => option.value === selected.value)
				if (index > 0) {
					selected = options[index - 1]
				}
			}
		} else if (event.key === 'ArrowDown') {
			if (selected) {
				const index = options.findIndex((option) => option.value === selected.value)
				if (index < options.length - 1) {
					selected = options[index + 1]
				}
			}
		} else if ((event.key === 'Escape' || event.key === 'Enter') && open) {
			open = false
		}
	}
</script>

<div
	class="select select--color-{color}"
	class:select--opens-up={false}
	use:clickOutside={() => {
		open = false
	}}
	bind:this={element}
	tabindex="0"
	on:keydown={keydown}
	on:click>
	<div
		class="select__input"
		on:click={() => {
			open = !open
		}}>
		{#if icon}
			<svelte:component this={icon} />
		{/if}
		<span class="select__input__value" style:min-width="{minWidth + 16 + 8}px">
			{label || selected?.label || value || 'Choose...'}
		</span>
		<div class="select__input__arrow">
			<slot name="expandIcon">
				<IconChevronDown />
			</slot>
		</div>
	</div>
	{#if open}
		<ul class="select__options" style:--selected-index={options.indexOf(selected)}>
			{#each options as option (option.value)}
				<li
					on:click={() => {
						selected = option
						open = false
					}}
					class:is-selected={selected?.value === option.value}>
					{option.label || option.value}
					{#if selected?.value === option.value}
						<IconCheck />
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style lang="postcss">
	.select {
		position: relative;
		display: flex;
		flex-direction: column;
		color: var(--color-button-text);
		cursor: pointer;
		border-radius: var(--rounded);
		align-self: flex-start;

		&__input {
			display: flex;
			width: 100%;
			justify-content: space-between;
			align-items: center;
			padding: 0.25rem 1rem;
			gap: 0.5rem;
			background-color: var(--color-button-bg);
			box-shadow: var(--shadow-inset-sm);
			border-radius: var(--rounded);
		}

		&__options {
			list-style-type: none;
			display: flex;
			flex-direction: column;
			width: 100%;
			padding: 0;
			margin: 0;
			position: absolute;
			top: calc(100% * -1 * var(--selected-index));
			background-color: var(--color-button-bg);
			border-radius: var(--rounded);
			box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
			border: var(--border-width) solid var(--color-tertiary);
			overflow: hidden;
			z-index: 5;

			li {
				padding: 0.25rem 1rem;

				&:hover {
					background-color: var(--color-brand-dark);
					color: var(--color-brand-dark-contrast);
				}

				&.is-selected {
					background-color: var(--color-brand-light);
					color: var(--color-text);
					cursor: default;
					display: flex;
					align-items: center;
					gap: 0.5rem;

					:global(.icon) {
					}
				}
			}
		}

		&--color-raised {
			> * {
				background-color: var(--color-raised-bg);
			}
		}

		&--opens-up {
			.select__options {
				bottom: 100%;
				top: auto;
				border-radius: var(--rounded-top);
				box-shadow: none;
				border-top: none;
				border-bottom: var(--border-width) solid var(--color-divider);
			}
		}
	}
</style>
