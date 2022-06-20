<script lang="ts">
	import { fade, fly } from 'svelte/transition'
	import Button from './Button.svelte'
	import { classCombine } from '../utils/classCombine'
	import IconX from 'virtual:icons/heroicons-outline/x'

	export let open = false

	/** Set the width of the modal */
	export let size: 'sm' | 'md' | 'lg' = 'md'

	export let title = ''

	/** If enabled, clicking outside the modal with close it */
	export let dismissable = true

	export let defaultData: Record<string, any> = {}
	export let data: Record<string, any> = defaultData

	function close() {
		open = false
		data = defaultData
	}

	function trigger() {
		open = !open
	}
</script>

<slot name="trigger" {trigger} />

{#if open}
	<div
		class="modal-background"
		transition:fade={{ duration: 300 }}
		on:click={() => {
			if (dismissable) close()
		}} />

	<div
		class={classCombine(['modal', `modal--size-${size}`, 'card'])}
		transition:fly={{ y: 400, duration: 250 }}>
		{#if title}
			<div class="modal__top">
				<h2 class="title-secondary">{title}</h2>
				<Button title="Close" color="transparent" on:click={close}>
					<IconX width={20} />
				</Button>
			</div>
		{/if}

		<slot />

		<div class="modal__buttons">
			<Button on:click={close}><IconX /> Cancel</Button>
			<slot name="button" {close} />
		</div>
	</div>
{/if}

<style lang="postcss">
	:global(.base.theme-dark > .modal, .base.theme-oled > .modal) {
		border: 1px solid var(--color-divider);
	}

	.modal-background {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background: hsla(0, 0%, 0%, 0.5);
		backdrop-filter: blur(3px);
		z-index: 20;
	}

	.modal {
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);

		width: 80%;
		z-index: 21;
		max-height: calc(100% - 2rem);
		overflow-y: auto;

		&--size {
			&-sm {
				max-width: 450px;
			}
			&-md {
				max-width: 600px;
			}
			&-lg {
				max-width: 750px;
			}
		}

		&__top {
			background-color: var(--color-bg);
			margin: -1rem -1rem 0.5rem -1rem;
			padding: 1rem 1rem 1rem 1.5rem;
			display: flex;
			align-items: center;
			justify-content: space-between;
		}

		&__danger {
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

		:global(p),
		:global(ul),
		:global(ol) {
			line-height: 1.5;
			margin: 0;

			:global(a) {
				color: var(--color-link);

				&:hover {
					text-decoration: underline;
				}
			}
		}

		&__buttons {
			margin-top: 1rem;
			display: flex;
			justify-content: flex-end;
			gap: 1rem;
			flex-wrap: wrap;
		}
	}
</style>
