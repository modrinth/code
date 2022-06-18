<script lang="ts">
	import IconChevronDown from 'virtual:icons/lucide/chevron-down'
	import IconCheck from 'virtual:icons/heroicons-outline/check'
	import { clickOutside } from 'svelte-use-click-outside'
	import { fade } from 'svelte/transition'

	interface Option {
		label: string
		value: string | number
	}

	export let options: Option[] = []
	export let value: string | number
	export let selected: Option | undefined
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

	let element: HTMLElement

	function selectOption(option: Option) {
		selected = option
		open = false
		element.focus()
	}

	function keydown(event: KeyboardEvent) {
		if (event.key === ' ' || event.key === 'Enter') {
			event.preventDefault()

			if (!open) {
				open = true
				// Needs delay before trying to move focus
				setTimeout(() => (element.children[1].children[0] as HTMLButtonElement).focus(), 0)
			} else {
				const option = options.find(
					({ label }) => label === document.activeElement.innerHTML.trim()
				)
				selectOption(option)
				open = false
			}
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
			open = true
		}}>
		{#if icon}
			<svelte:component this={icon} />
		{/if}
		<span class="select__input__value" style:min-width="{minWidth}px">
			{label || selected?.label || value || 'Choose...'}
		</span>
		<div class="select__input__arrow">
			<slot name="expandIcon">
				<IconChevronDown />
			</slot>
		</div>
	</div>
	{#if open}
		<div
			transition:fade={{ duration: 70 }}
			class="select__options"
			style:--selected-index={options.indexOf(selected)}>
			{#each options as option, index (option.value)}
				{@const isSelected = selected?.value === option.value}
				<button
					on:click={() => selectOption(option)}
					class:is-selected={isSelected}
					tabindex={isSelected ? -1 : 0}
					on:focusout={() => {
						if (index + 1 === options.length) open = false
					}}>
					{option.label || option.value}
					{#if selected?.value === option.value}
						<IconCheck />
					{/if}
				</button>
			{/each}
		</div>
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
			box-shadow: var(--shadow-inset-sm), var(--shadow-floating), 0 0 0 1px var(--color-tertiary);
			/* border: var(--border-width) solid var(--color-tertiary); */
			overflow: hidden;
			z-index: 5;

			button {
				padding: 0.25rem 1rem;
				display: flex;
				align-items: center;
				gap: 0.5rem;

				&:hover,
				&:focus {
					background-color: var(--color-brand-dark);
					color: var(--color-brand-dark-contrast);
					outline: none;
					border-radius: 0;
				}

				&.is-selected {
					background-color: var(--color-brand-light);
					color: var(--color-text);
					cursor: default;
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
