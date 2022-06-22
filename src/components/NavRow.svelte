<script lang="ts">
	import { prerendering } from '$app/env'
	import { page } from '$app/stores'
	import { onMount } from 'svelte'

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

	$: activeIndex = query
		? prerendering
			? -1
			: links.findIndex((link) => ($page.url.searchParams.get(query) || '') === link.href)
		: links.findIndex((link) => path[level] === link.href || path[level] === link.href.slice(0, -1))

	const linkElements: HTMLAnchorElement[] = []

	let indicatorReady = false

	onMount(() => {
		setTimeout(() => {
			indicatorReady = true
		}, 300)
	})
</script>

<nav class="navigation" class:static-indicator={!indicatorReady}>
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
		style:visibility={indicatorReady && activeIndex !== -1 ? 'visible' : 'hidden'}
		style:left={linkElements
			.slice(0, activeIndex)
			.map((element, index) => element.offsetWidth + 16)
			.reduce((acc, a) => acc + a, 0) + 'px'}
		style:width={linkElements[activeIndex]?.offsetWidth + 'px'} />
</nav>

<style lang="postcss">
	.navigation {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 1rem;
		flex-wrap: wrap;
		position: relative;

		&__link {
			font-weight: var(--font-weight-bold);
			color: var(--color-text-light);
			position: relative;

			&.is-active {
				color: var(--color-text);
			}

			&::after {
				content: '';
				display: block;
				position: absolute;
				bottom: -2px;
				width: 100%;
				border-radius: var(--rounded-max);
				height: 0.25rem;
				transition: opacity 0.2s ease-in-out;
				background-color: var(--color-brand);
				opacity: 0;
			}

			&:hover::after {
				opacity: 0.5;
			}
		}

		&.static-indicator {
			.navigation__link {
				&.is-active::after {
					background-color: var(--color-brand);
				}
			}
		}

		&__indicator {
			position: absolute;
			bottom: -2px;
			width: 1rem;
			height: 0.25rem;
			border-radius: var(--rounded-max);
			background-color: var(--color-brand);
			transition: width 0.3s ease-out, left 0.3s ease-out;
			visibility: hidden;
		}
	}
</style>
