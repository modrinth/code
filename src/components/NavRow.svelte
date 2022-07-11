<script lang="ts">
	import { browser, prerendering } from '$app/env'
	import { page } from '$app/stores'
	import { onDestroy, onMount } from 'svelte'
	import { debounce } from 'throttle-debounce'

	interface Link {
		href: string
		label: string
	}

	export let links: Link[]
	/** Query param name to use (to not use queries, leave this prop blank) */
	export let query = ''

	export let resetScroll = false

	/** Path level in URL, zero-indexed */
	export let level = 0

	let path: string[]
	$: path = [
		...$page.url.pathname
			.substring(1)
			.split('/')
			.map((route) => '/' + route),
		'/',
	]

	$: basePath = path.slice(0, level).join('')

	/* Animation */
	const FAST_TIMING = 'cubic-bezier(1,0,.3,1) -140ms'
	const SLOW_TIMING = 'cubic-bezier(.75,-0.01,.24,.99) -40ms'

	$: activeIndex = query
		? prerendering
			? -1
			: links.findIndex((link) => ($page.url.searchParams.get(query) || '') === link.href)
		: links.findIndex((link) => path[level] === link.href || path[level] === link.href.slice(0, -1))

	let oldIndex = -1

	const linkElements: HTMLAnchorElement[] = []

	const indicator = {
		left: 0,
		right: 0,
		top: 22,
		direction: 'right',
	}

	$: if (activeIndex > -1 && browser && linkElements.length > 0) startAnimation()

	function startAnimation() {
		// Avoids error that `linkElements[activeIndex]` is null
		if (linkElements[activeIndex]) {
			indicator.direction = activeIndex < oldIndex ? 'left' : 'right'

			indicator.left = linkElements[activeIndex].offsetLeft
			indicator.right =
				linkElements[activeIndex].parentElement.offsetWidth -
				linkElements[activeIndex].offsetLeft -
				linkElements[activeIndex].offsetWidth
			indicator.top =
				linkElements[activeIndex].offsetTop + linkElements[activeIndex].offsetHeight - 2
		}

		oldIndex = activeIndex
	}

	const debounced = debounce(100, startAnimation)

	let useAnimation = false

	onMount(() => {
		setTimeout(() => {
			useAnimation = true
		}, 300)

		window.addEventListener('resize', debounced)
	})

	onDestroy(() => {
		if (browser) window.removeEventListener('resize', debounced)
	})
</script>

<nav class="navigation" class:use-animation={useAnimation}>
	{#each links as link, index}
		<a
			href={query
				? link.href
					? `?${query}=${link.href}`
					: '?'
				: level === 0
				? link.href
				: basePath + link.href}
			on:click={() => {
				if (resetScroll) document.body.scrollTo(0, 0)
			}}
			class="navigation__link"
			class:is-active={index === activeIndex}
			sveltekit:noscroll={!resetScroll || null}
			bind:this={linkElements[index]}>
			{link.label}
		</a>
	{/each}
	<div
		class="navigation__indicator"
		style:visibility={useAnimation && activeIndex !== -1 ? 'visible' : 'hidden'}
		style:left={indicator.left + 'px'}
		style:right={indicator.right + 'px'}
		style:top={indicator.top + 'px'}
		style:transition={`left 350ms ${
			indicator.direction === 'left' ? FAST_TIMING : SLOW_TIMING
		},right 350ms ${
			indicator.direction === 'right' ? FAST_TIMING : SLOW_TIMING
		}, top 100ms ease-in-out`} />
</nav>

<style lang="postcss">
	.navigation {
		display: flex;
		flex-direction: row;
		align-items: center;
		grid-gap: 1rem;
		flex-wrap: wrap;
		position: relative;

		&__link {
			font-weight: var(--font-weight-bold);
			color: var(--color-text-light);
			position: relative;

			&::after {
				content: '';
				display: block;
				position: absolute;
				bottom: -2px;
				width: 100%;
				border-radius: var(--rounded-max);
				height: 0.25rem;
				transition: opacity 0.1s ease-in-out;
				background-color: var(--color-brand);
				opacity: 0;
			}

			&:hover {
				color: var(--color-text);

				&::after {
					opacity: 0.4;
				}
			}

			&:active::after {
				opacity: 0.2;
			}

			&.is-active {
				color: var(--color-text);

				&::after {
					opacity: 1;
				}
			}
		}

		&.use-animation {
			.navigation__link {
				&.is-active::after {
					opacity: 0;
				}
			}
		}

		&__indicator {
			position: absolute;
			height: 0.25rem;
			border-radius: var(--rounded-max);
			background-color: var(--color-brand);
			transition-property: left, right, top;
			transition-duration: 350ms;
			visibility: hidden;
		}
	}
</style>
