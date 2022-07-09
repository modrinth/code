<script lang="ts">
	import IconChevronDown from 'virtual:icons/lucide/chevron-down'
	import IconCheck from 'virtual:icons/heroicons-outline/check'
	import { clickOutside } from 'svelte-use-click-outside'
	import { onDestroy, onMount, tick } from 'svelte'
	import { debounce } from 'throttle-debounce'
	import { browser } from '$app/env'

	interface Option {
		label: string
		value: string | number
	}

	export let options: Option[] = []
	export let value: string | number
	export let color = ''
	export let label = ''
	export let icon = null

	let selected: Option | undefined
	let open = false
	let direction = 'down'
	let checkingDirection = false
	let element: HTMLDivElement

	$: if (selected) value = selected.value

	$: if (options) setSelected()

	function setSelected() {
		selected = options.find((option) => option.value === (value || ''))
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
				return text.length * 8.3
			}
		},
		// Reuses the same canvas object
		{ canvas: null }
	)

	const minWidth = Math.max(
		...options.map((it) => getTextWidth(String(it.label || it.value), '16px Inter')),
		...(!value ? [71] : []) // width of "Choose..." text
	)

	function selectOption(option: Option) {
		selected = option
		open = false
		element.blur()
	}

	// Checks if there is enough room below the element to show the dropdown, if not, show above the element
	async function checkDirection() {
		checkingDirection = true
		await tick()
		const { bottom } = element.children[0].getBoundingClientRect()
		const height = (element.children[1] as HTMLDivElement).offsetHeight
		const windowBottom = window.scrollY + window.innerHeight
		const spaceBelow = windowBottom - bottom
		if (spaceBelow < height) {
			direction = 'up'
		} else {
			direction = 'down'
		}
		checkingDirection = false
	}

	function keydown(event: KeyboardEvent) {
		const currentIndex = options.indexOf(selected)

		if (event.key === 'End') {
			selected = options[options.length - 1]
		} else if (['ArrowDown', 'ArrowUp'].includes(event.key)) {
			event.preventDefault()
			if (!open) {
				open = true
				return
			}

			if (
				(event.key === 'ArrowUp' && direction === 'down') ||
				(event.key === 'ArrowDown' && direction === 'up')
			) {
				if (currentIndex > 0) {
					selected = options[currentIndex - 1]
				} else {
					selected = options[options.length - 1]
				}
			} else if (
				(event.key === 'ArrowDown' && direction === 'down') ||
				(event.key === 'ArrowUp' && direction === 'up')
			) {
				if (currentIndex < options.length - 1) {
					selected = options[currentIndex + 1]
				} else {
					selected = options[0]
				}
			}
		} else if (event.key === 'Home') {
			selected = options[0]
		} else if (event.key === 'Escape') {
			if (open) {
				// prevent ESC bubble in this case (interfering with modal closing etc)
				event.preventDefault()
				event.stopPropagation()

				open = false
			}
		} else if (['Enter', ' '].includes(event.key)) {
			if (!open) {
				open = true
			} else {
				open = false
			}
			if (event.key === 'Enter' || event.key === ' ') {
				event.preventDefault()
			}
		}
	}

	const debounced = debounce(100, checkDirection)

	onMount(() => {
		checkDirection()

		window.addEventListener('resize', debounced)
		document.body.addEventListener('scroll', debounced)
	})

	onDestroy(() => {
		if (browser) {
			window.removeEventListener('resize', debounced)
			document.body.removeEventListener('scroll', debounced)
		}
	})
</script>

<div
	class="select select--color-{color}"
	use:clickOutside={() => (open = false)}
	bind:this={element}
	tabindex="0"
	on:focus={() => (open = true)}
	on:blur={() => (open = false)}
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
	<div
		class="select__options select__options--direction-{direction}"
		class:open
		style:max-height={checkingDirection ? 'unset' : null}>
		<button class="select__options__item current" on:click={() => (open = false)} tabindex="-1">
			{#if icon}
				<svelte:component this={icon} />
			{/if}
			<span>{label || selected?.label || value || 'Choose...'}</span>
			<IconChevronDown class="icon-chevron" />
		</button>
		{#each options as option, index (option.value)}
			{@const isSelected = selected?.value === option.value}
			<button
				class="select__options__item"
				on:click={() => selectOption(option)}
				class:is-selected={isSelected}
				tabindex="-1"
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
		user-select: none;

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
			display: flex;
			width: 100%;
			padding: 0;
			margin: 0;
			position: absolute;
			background-color: var(--color-button-bg);
			border-radius: var(--rounded);
			box-shadow: var(--shadow-inset-sm), var(--shadow-floating), 0 0 0 1px var(--color-tertiary);
			overflow: hidden;
			z-index: 5;
			visibility: hidden;
			max-height: 32px;
			transition: max-height 0.3s ease-in-out, visibility 0.3s ease-in-out;

			&.open {
				visibility: visible;
				max-height: 100vh;

				.select__options__item.current :global(.icon-chevron) {
					transform: rotate(180deg);
				}
			}

			&--direction {
				&-down {
					flex-direction: column;
				}

				&-up {
					flex-direction: column-reverse;
					bottom: 0;
				}
			}

			&__item {
				padding: 0.25rem 1rem;
				display: flex;
				align-items: center;
				gap: 0.5rem;

				&:hover:not(.current, .is-selected) {
					background-color: var(--color-brand-dark);
					color: var(--color-brand-dark-contrast);
					outline: none;
					border-radius: 0;
				}

				&.current {
					box-shadow: 0 0 0 1px var(--color-tertiary);

					:global(.icon-chevron) {
						margin-left: auto;
						margin-top: 0.2rem;
						transition: transform 0.2s ease-in-out;
					}
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
