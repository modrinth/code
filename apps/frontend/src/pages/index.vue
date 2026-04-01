<template>
	<div class="px-4 py-6">
		<div class="grid grid-cols-[2fr,1fr] gap-3">
			<div class="pb-4">
				<span class="text-3xl">Welcome to Modrinth</span>
				<p
					class="flex items-center gap-2 border border-solid border-yellow-600 bg-yellow-100 p-3 italic"
				>
					<HelpFaqIcon class="size-6 brightness-[0.9]" /> {{ tagline }}
				</p>
				<div class="grid grid-cols-3 gap-2">
					<div class="border border-solid border-[#96CEE0] bg-[#E6F1F5] p-3">
						Publish your Minecraft mods and earn rewards!
						<button
							type="button"
							class="m-0 mt-2 block w-fit bg-transparent p-0 text-link"
							@click="onCreateProjectClick($event)"
						>
							Create a project ►
						</button>
					</div>
					<div class="block border border-solid border-[#96CEE0] bg-[#E6F1F5] p-3">
						Find mods to download and install
						<nuxt-link to="/mods" class="mt-2 block w-fit text-link"> Browse ►</nuxt-link>
					</div>
					<div class="border border-solid border-[#96CEE0] bg-[#E6F1F5] p-3">
						Play with your friends on our hosting service!
						<nuxt-link to="/hosting" class="mt-2 block w-fit text-link"> Host a server ►</nuxt-link>
					</div>
				</div>
				<div class="mt-2 border border-solid border-[#c3c3c3] bg-[#EFEFEF] p-4">
					<p class="m-0 text-xl font-bold">Modrinth is better with the desktop application!</p>
					<p class="m-0 mt-2">
						The easiest way to manage multiple mod installations, create modpacks, and more!
					</p>
					<div class="mt-3 flex">
						<ButtonStyled color="brand" size="large">
							<nuxt-link to="/app">Download!</nuxt-link>
						</ButtonStyled>
					</div>
				</div>
				<div v-if="featuredProjects.length" class="relative mt-4">
					<p class="m-0 mb-2 text-xl font-bold">Featured by our staff</p>
					<div class="flex">
						<img
							src="/frog.png"
							alt="Mr. Frog pointing at the featured staff picks"
							class="h-[300px] w-[250px] object-cover object-center"
						/>
						<div class="grid grid-cols-2 gap-2">
							<div
								v-for="project in featuredProjects"
								:key="project.id"
								class="border border-solid border-[#c3c3c3] bg-[#EFEFEF] p-3"
							>
								<div class="flex gap-2">
									<Avatar
										:src="project.icon_url"
										size="40px"
										class="shrink-0 rounded-[4px]"
										no-shadow
									/>
									<div class="min-w-0">
										<nuxt-link
											:to="`/${project.project_type}/${project.slug ?? project.id}`"
											class="font-semibold text-link"
										>
											{{ project.title }}
										</nuxt-link>
										<p class="m-0 mt-1 line-clamp-2 text-sm">{{ project.description }}</p>
									</div>
								</div>
							</div>
						</div>
					</div>
					<nuxt-link
						v-if="featuredCollection"
						:to="`/collection/${featuredCollection.id}`"
						class="ml-auto mt-3 block w-fit text-link"
					>
						View more featured projects ►
					</nuxt-link>
				</div>
			</div>
			<div class="flex h-full">
				<LatestNewsRow class="h-full" />
			</div>
		</div>
		<div
			class="mt-2 grid grid-cols-[1fr,auto] border border-solid border-[#ae96e0] bg-[#f3e6f5] p-4"
		>
			<div>
				<p class="m-0 text-xl font-bold">Say good-bye to ads with Modrinth+</p>
				<p class="m-0 mt-2">
					Help support the development of Modrinth and its creators by subscribing to Modrinth+!
				</p>
			</div>
			<div class="mt-3 flex">
				<ButtonStyled color="purple" size="large">
					<nuxt-link to="/plus">Subscribe!</nuxt-link>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { HelpFaqIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, inject } from 'vue'

import LatestNewsRow from '~/components/ui/news/LatestNewsRow.vue'

const FEATURED_COLLECTION_ID = 'YV97U1kk'
const PROJECT_FETCH_CHUNK = 800

function shuffleInPlace<T>(items: T[]): T[] {
	for (let i = items.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1))
		const t = items[i]!
		items[i] = items[j]!
		items[j] = t
	}
	return items
}

const client = injectModrinthClient()

const { data: featuredPayload } = useQuery({
	queryKey: ['home', 'featured-from-collection', FEATURED_COLLECTION_ID],
	queryFn: async () => {
		const collection = await client.labrinth.collections.get(FEATURED_COLLECTION_ID)
		const projectIds = collection.projects ?? []
		if (projectIds.length === 0) {
			return { collection, projects: [] as Labrinth.Projects.v2.Project[] }
		}
		const segments: string[][] = []
		for (let i = 0; i < projectIds.length; i += PROJECT_FETCH_CHUNK) {
			segments.push(projectIds.slice(i, i + PROJECT_FETCH_CHUNK))
		}
		const results = await Promise.all(
			segments.map((ids) => client.labrinth.projects_v2.getMultiple(ids)),
		)
		const projects = results.flat()
		for (const project of projects) {
			project.categories = project.categories.concat(project.loaders)
		}
		const sample = shuffleInPlace([...projects]).slice(0, 6)
		return { collection, projects: sample }
	},
	staleTime: 1000 * 60 * 10,
	retry: false,
})

const featuredProjects = computed(() => featuredPayload.value?.projects ?? [])
const featuredCollection = computed(() => featuredPayload.value?.collection ?? null)

const auth = await useAuth()
const route = useRoute()

const openProjectCreateModal =
	inject<(event?: MouseEvent, options?: { type?: 'server' | 'project' }) => void>(
		'openProjectCreateModal',
	)

function onCreateProjectClick(event: MouseEvent) {
	if (!auth.value.user) {
		navigateTo(`/auth/sign-in?redirect=${encodeURIComponent(route.fullPath)}`)
		return
	}
	openProjectCreateModal?.(event)
}

const TAGLINES = [
	'The ultimate platform for modding Minecraft!',
	'The everything application!',
	`We're sorry, Singapore!`,
	'Ribbit',
	'Never gonna give you up!',
	'Buy with beenz!',
	'Best viewed in Netscape Navigator!',
	'Over a million devices run Modrinth!',
	'Happy new year!',
	'Not to be confused with CurseForge',
	'Better than Curse Client!',
	`It's open source, fix it yourself!`,
	'Gentlemen, this is democracy manifest!',
	'Got any grapes?',
	'Thanks, Obama.',
	'Is this loss?',
	'FUUUUUUU',
	'Move away from the mic to breathe in!',
	'Spaghetti!',
	"Lamp oil, rope, bombs? You want it? It's yours, my friend.",
	"It's over 9000!",
	'Good guys, bad guys and explosions, as far as the eye can see!',
	"I'm doing my part!",
]

function pickWeightedTagline(taglines: string[]): string {
	if (taglines.length === 0) return ''
	if (taglines.length === 1) return taglines[0] ?? ''

	const r = Math.random()
	if (r < 0.5) return taglines[0] ?? ''

	const rest = taglines.slice(1)
	const idx = Math.floor(((r - 0.5) / 0.5) * rest.length)
	return rest[Math.min(Math.max(idx, 0), rest.length - 1)] ?? taglines[0] ?? ''
}

const tagline = useState('landing-tagline', () => pickWeightedTagline(TAGLINES))
</script>
