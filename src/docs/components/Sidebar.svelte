<script lang="ts">
	import { page } from '$app/stores'

	import IconMenu from 'virtual:icons/lucide/menu'

	const components = Object.keys(import.meta.glob('../../routes/components/**'))
		.map((it) => it.replace('../../routes/components/', '').replace('.md', ''))
		.sort()

	const classes = Object.keys(import.meta.glob('../../routes/classes/**'))
		.map((it) => it.replace('../../routes/classes/', '').replace('.md', ''))
		.sort()

	let slideIn = false

	$: if ($page.url.pathname) {
		slideIn = false
	}
</script>

<nav class="sidebar" class:slideIn>
	<div class="sidebar__content">
		<div class="section">
			<span class="section__title">Getting started</span>
			<a href="/" class="section__link">Introduction</a>
			<a href="/getting-started/configure" class="section__link">Configure</a>
			<a href="/getting-started/icons" class="section__link">Using Icons</a>
			<a href="/getting-started/css" class="section__link">Writing CSS</a>
			<a href="/getting-started/illustrations" class="section__link">Illustrations</a>
			<a href="/getting-started/utils" class="section__link">Built-in utilities</a>
			<a href="/getting-started/generator" class="section__link">Generator plugin</a>
		</div>

		<div class="section">
			<span class="section__title">Components</span>
			{#each components as component}
				<a href="/components/{component}" class="section__link">{component}</a>
			{/each}
		</div>

		<div class="section">
			<span class="section__title">Classes</span>
			{#each classes as page}
				<a href="/classes/{page}" class="section__link">{page}</a>
			{/each}
		</div>
	</div>

	<button class="sidebar__toggle" on:click={() => (slideIn = !slideIn)}>
		<IconMenu />
	</button>
</nav>

<style lang="postcss">
	:root {
		--sidebar-color: hsl(220, 15%, 40%);
		--title-color: hsl(216, 10%, 80%);
		--link-color: hsl(216, 10%, 90%);
		--scrollbar-thumb-color: hsl(216, 10%, 70%);
	}

	.sidebar {
		background-color: var(--sidebar-color);
		color: var(--title-color);
		width: var(--sidebar-width);
		max-width: 70vw;
		position: fixed;
		left: -100%;
		top: 0;
		z-index: 5;
		transition: left 0.2s ease-in-out;
		box-shadow: 2px 0px 4px hsla(221, 39%, 11%, 0.2);

		@media (--md) {
			left: 0;
		}

		&__content {
			mask-image: linear-gradient(to bottom, transparent, hsla(0, 0%, 0%, 1) 5% 95%, transparent);
			padding: 88px 2rem 2rem;
			height: 100vh;
			max-height: 100vh;
			overflow-y: auto;
			gap: 2.5rem;
			display: flex;
			flex-direction: column;

			.section {
				display: flex;
				flex-direction: column;
				gap: 0.5rem;

				&__title {
					text-transform: uppercase;
					font-size: 12px;
				}

				&__link {
					color: var(--link-color);
					text-decoration: none;

					&:hover {
						color: white;
						text-decoration: underline;
					}
				}
			}
		}

		&__toggle {
			position: fixed;
			left: 1rem;
			bottom: 1rem;
			padding: 0.5rem;
			aspect-ratio: 1 / 1;
			background-color: var(--accent-color);
			z-index: 20;
			border-radius: var(--rounded);
			color: white;
			box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
			transition: left 0.2s cubic-bezier(0.38, 0.52, 0.37, 1.27);

			:global(.icon) {
				width: 2rem;
				height: auto;
			}

			@media (--md) {
				visibility: hidden;
			}
		}

		&.slideIn {
			left: 0;

			.sidebar__toggle {
				left: calc(2rem + min(70vw, var(--sidebar-width)));
			}
		}

		scrollbar-color: var(--scrollbar-thumb-color) var(--sidebar-color);

		&::-webkit-scrollbar {
			width: 1rem;
		}

		&::-webkit-scrollbar-track {
			background-color: var(--sidebar-color);
		}

		&::-webkit-scrollbar-thumb {
			background-color: var(--scrollbar-thumb-color);
			border-radius: var(--rounded-max);
			border: 3px solid var(--sidebar-color);
		}
	}
</style>
