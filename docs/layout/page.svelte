<script lang="ts">
	import IconPencil from 'virtual:icons/heroicons-outline/pencil'
	import { page } from '$app/stores'
	import COMPONENT_API from '../../generated/COMPONENT_API.json'
	import { markdownInline } from 'omorphia/utils'

	export let fileName = $page.url.pathname
		.substring($page.url.pathname.lastIndexOf('/') + 1)
		.replace('.html', '')

	export let title = fileName

	export let description = 'Learn about Omorphia, the component & style library'

	let editUrl = `https://github.com/modrinth/omorphia/edit/main/docs/routes/${
		$page.url.pathname.replace('/', '') || 'index'
	}.md`

	let api = { props: [], events: [], slots: [] }
	if ($page.url.pathname.includes('components')) {
		if (import.meta.env.DEV) {
			import(`../../src/components/${title}.svelte?raw&sveld`).then(
				(output) => (api = output.default)
			)
		} else {
			api = COMPONENT_API[`${title}.svelte`]
		}
	}
</script>

<svelte:head>
	<title>{title ? `${title} • Omorphia` : 'Omorphia'}</title>
	<meta name="description" content={description} />
</svelte:head>

{#if title}<h1>{title}</h1>{/if}
<a class="edit-link" href={editUrl}>
	<IconPencil />
	Edit this page on GitHub</a>
<slot />

{#if api}
	<div class="extra-info">
		{#if api.props.length > 0}
			<h2>Properties</h2>
			<table class="api-table">
				<thead>
					<tr>
						<th>Name</th>
						<th>Type</th>
						<th>Default</th>
						<th>Description</th>
					</tr>
				</thead>
				<tbody>
					{#each api.props as prop}
						<tr>
							<td><code>{prop.name}</code></td>
							<td><code>{prop.type ?? ''}</code></td>
							<td><code>{prop.value ?? ''}</code></td>
							<td>
								{prop.constant ? '[Read only] ' : ''}
								{@html markdownInline(prop.description?.replace('null', '') || '')}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
		{#if api.events.length > 0}
			<h2>Events</h2>
			<table class="api-table">
				<thead>
					<tr>
						<th>Name</th>
						<th>Forwarded</th>
						<th>Description</th>
					</tr>
				</thead>
				<tbody>
					{#each api.events as event}
						<tr>
							<td><code>{event.name}</code></td>
							<td>{!!event.parent}</td>
							<td>{event.description?.replace('null', '') || ''}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
		{#if api.slots.length > 0}
			<h2>Slots</h2>
			<table class="api-table">
				<thead>
					<tr>
						<th>Name</th>
						<th>Fallback</th>
					</tr>
				</thead>
				<tbody>
					{#each api.slots as slot}
						<tr>
							<td><code>{slot.name}</code></td>
							<td>{slot.fallback ?? 'None'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
{/if}

<style lang="postcss">
	.edit-link {
		display: flex;
		align-items: center;
		grid-gap: 0.5rem;
		margin-bottom: 54px;
		color: var(--accent-color);
	}

	.extra-info {
		margin-top: 4rem;
	}

	.api-table {
		border-collapse: collapse;
		margin-top: -6px;
	}

	.api-table tr {
		background-color: transparent;
		border: none;
	}

	.api-table tbody {
		border: 2px solid grey;
	}

	.api-table th {
		text-transform: uppercase;
		font-size: 12.5px;
		border: none;
	}
</style>
