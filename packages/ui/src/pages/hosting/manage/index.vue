<template>
	<div
		data-pyro-server-list-root
		class="experimental-styles-within relative mx-auto mb-6 flex min-h-screen w-full max-w-[1280px] flex-col px-6"
	>
		<ServersUpgradeModalWrapper
			v-if="isNuxt"
			ref="upgradeModal"
			:stripe-publishable-key
			:site-url
			:products
		/>

		<div
			v-if="hasError || fetchError"
			class="mx-auto flex h-full min-h-[calc(100vh-4rem)] flex-col items-center justify-center gap-4 text-left"
		>
			<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
				<div class="flex flex-col items-center text-center">
					<div class="flex flex-col items-center gap-4">
						<div class="grid place-content-center rounded-full bg-bg-blue p-4">
							<HammerIcon class="size-12 text-blue" />
						</div>
						<h1 class="m-0 w-fit text-3xl font-bold">Servers could not be loaded</h1>
					</div>
					<p class="text-lg text-secondary">We may have temporary issues with our servers.</p>
					<ul class="m-0 list-disc space-y-4 p-0 pl-4 text-left text-sm leading-[170%]">
						<li>
							Our systems automatically alert our team when there's an issue. We are already working
							on getting them back online.
						</li>
						<li>
							If you recently purchased your Modrinth Hosting server, it is currently in a queue and
							will appear here as soon as it's ready. <br />
							<span class="font-medium text-contrast"
								>Do not attempt to purchase a new server.</span
							>
						</li>
						<li>
							If you require personalized support regarding the status of your server, please
							contact Modrinth Support.
						</li>

						<li v-if="fetchError" class="text-red">
							<p>Error details:</p>
							<CopyCode
								:text="(fetchError as ModrinthServersFetchError).message || 'Unknown error'"
								:copyable="false"
								:selectable="false"
								:language="'json'"
							/>
						</li>
					</ul>
				</div>
				<ButtonStyled size="large" type="standard" color="brand">
					<AutoLink class="mt-6 !w-full" to="https://support.modrinth.com"
						>Contact Modrinth Support</AutoLink
					>
				</ButtonStyled>
				<ButtonStyled size="large" @click="() => router.go(0)">
					<button class="mt-3 !w-full">Reload</button>
				</ButtonStyled>
			</div>
		</div>

		<Transition v-else name="fade" mode="out-in">
			<div v-if="isLoading && !serverResponse" key="loading" class="flex flex-col gap-4 py-8">
				<div class="mb-4 text-center">
					<LoaderCircleIcon class="mx-auto size-8 animate-spin text-contrast" />
					<p class="m-0 mt-2 text-secondary">Loading your servers...</p>
				</div>
				<div
					v-for="i in 3"
					:key="i"
					class="flex animate-pulse flex-row items-center gap-4 overflow-x-hidden rounded-2xl border-[1px] border-solid border-button-bg bg-bg-raised p-4"
				>
					<div class="size-16 rounded-xl bg-button-bg"></div>
					<div class="flex flex-1 flex-col gap-2">
						<div class="h-6 w-48 rounded bg-button-bg"></div>
						<div class="h-4 w-64 rounded bg-button-bg opacity-75"></div>
					</div>
				</div>
			</div>

			<div
				v-else-if="serverList.length === 0 && !isPollingForNewServers"
				key="empty"
				class="flex h-full flex-col items-center justify-center gap-8"
			>
				<img
					src="https://cdn.modrinth.com/servers/excitement.webp"
					alt=""
					class="max-w-[360px]"
					style="
						mask-image: radial-gradient(97% 77% at 50% 25%, #d9d9d9 0, hsla(0, 0%, 45%, 0) 100%);
					"
				/>
				<h1 class="m-0 text-contrast">You don't have any servers yet!</h1>
				<p class="m-0">Modrinth Hosting is a new way to play modded Minecraft with your friends.</p>
				<ButtonStyled size="large" type="standard" color="brand">
					<AutoLink to="/servers#plan">Create a server</AutoLink>
				</ButtonStyled>
			</div>

			<div v-else key="list">
				<div class="relative flex h-fit w-full flex-col items-center justify-between md:flex-row">
					<h1 class="w-full text-4xl font-bold text-contrast">Servers</h1>
					<div class="mb-4 flex w-full flex-row items-center justify-end gap-2 md:mb-0 md:gap-4">
						<div class="iconified-input w-full md:w-72">
							<label class="sr-only" for="search">Search</label>
							<SearchIcon />
							<input
								id="search"
								v-model="searchInput"
								class="input-text-inherit"
								type="search"
								name="search"
								autocomplete="off"
								placeholder="Search servers..."
							/>
						</div>
						<ButtonStyled v-if="isNuxt" type="standard">
							<AutoLink :to="{ path: '/servers', hash: '#plan' }">
								<PlusIcon />
								New server
							</AutoLink>
						</ButtonStyled>
					</div>
				</div>

				<Transition
					enter-active-class="transition-all duration-300 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-20"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-20"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="isPollingForNewServers"
						class="bg-brand/10 my-4 flex items-center justify-center gap-2 rounded-full px-4 py-2 text-sm text-brand"
					>
						<LoaderCircleIcon class="size-4 animate-spin" />
						<span>Checking for new servers...</span>
					</div>
				</Transition>

				<TransitionGroup
					v-if="filteredData.length > 0 || isPollingForNewServers"
					name="list"
					tag="ul"
					class="m-0 flex flex-col gap-4 p-0"
				>
					<MedalServerListing
						v-for="server in filteredData.filter((s) => s.is_medal)"
						:key="server.server_id"
						v-bind="server"
						@upgrade="openUpgradeModal(server.server_id)"
					/>
					<ServerListing
						v-for="server in filteredData.filter((s) => !s.is_medal)"
						:key="server.server_id"
						v-bind="server"
					/>
				</TransitionGroup>
				<div v-else class="flex h-full items-center justify-center">
					<p class="text-contrast"><LoaderCircleIcon class="size-5 animate-spin" /></p>
				</div>
			</div>
		</Transition>
	</div>
</template>

<script setup lang="ts">
import { type Archon, type Labrinth, NuxtModrinthClient } from '@modrinth/api-client'
import { HammerIcon, LoaderCircleIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import { AutoLink, ButtonStyled, CopyCode, injectModrinthClient } from '@modrinth/ui'
import type { ModrinthServersFetchError } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import Fuse from 'fuse.js'
import type { ComponentPublicInstance } from 'vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ServersUpgradeModalWrapper from '../../../components/billing/ServersUpgradeModalWrapper.vue'
import MedalServerListing from '../../../components/servers/marketing/MedalServerListing.vue'
import ServerListing from '../../../components/servers/ServerListing.vue'

defineProps<{
	stripePublishableKey?: string
	siteUrl?: string
	products?: Labrinth.Billing.Internal.Product[]
}>()

const router = useRouter()
const route = useRoute()
const client = injectModrinthClient()

const isNuxt = computed(() => client instanceof NuxtModrinthClient)

const hasError = ref(false)
const isPollingForNewServers = ref(false)
const pollingState = ref({
	enabled: false,
	count: 0,
	initialServers: [] as Archon.Servers.v0.Server[],
})

const {
	data: serverResponse,
	error: fetchError,
	isLoading,
} = useQuery({
	queryKey: ['servers'],
	queryFn: async () => {
		const response = await client.archon.servers_v0.list()

		// Fetch subscriptions for medal servers
		const hasMedalServers = response.servers.some((s) => s.is_medal)
		if (hasMedalServers) {
			const subscriptions = await client.labrinth.billing_internal.getSubscriptions()

			// Inject medal_expires into servers
			for (const server of response.servers) {
				if (server.is_medal) {
					const sub = subscriptions.find((s) => s.metadata?.id === server.server_id)
					if (sub) {
						server.medal_expires = dayjs(sub.created).add(5, 'days').toISOString()
					}
				}
			}
		}

		// Check if new servers appeared (stop polling)
		if (pollingState.value.enabled) {
			pollingState.value.count++
			if (response.servers.length !== pollingState.value.initialServers.length) {
				pollingState.value.enabled = false
				router.replace({ query: {} })
			} else if (pollingState.value.count >= 5) {
				pollingState.value.enabled = false
			}
		}

		return response
	},
	refetchInterval: () => (pollingState.value.enabled ? 5000 : false),
})

watch([fetchError, serverResponse], ([error, response]) => {
	hasError.value = !!error || !response
})

const serverList = computed<Archon.Servers.v0.Server[]>(() => {
	if (!serverResponse.value) return []
	return serverResponse.value.servers
})

const searchInput = ref('')

const fuse = computed(() => {
	if (serverList.value.length === 0) return null
	return new Fuse(serverList.value, {
		keys: ['name', 'loader', 'mc_version', 'game', 'state'],
		includeScore: true,
		threshold: 0.4,
	})
})

function introToTop(array: Archon.Servers.v0.Server[]): Archon.Servers.v0.Server[] {
	return array.slice().sort((a, b) => {
		return Number(b.flows?.intro) - Number(a.flows?.intro)
	})
}

const filteredData = computed<Archon.Servers.v0.Server[]>(() => {
	if (!searchInput.value.trim()) {
		return introToTop(serverList.value)
	}
	return fuse.value
		? introToTop(fuse.value.search(searchInput.value).map((result) => result.item))
		: []
})

onMounted(() => {
	if (route.query.redirect_status === 'succeeded') {
		isPollingForNewServers.value = true
		pollingState.value = {
			enabled: true,
			count: 0,
			initialServers: [...(serverResponse.value?.servers ?? [])],
		}
	}
})

type ServersUpgradeModalWrapperRef = ComponentPublicInstance<{
	open: (id: string) => void | Promise<void>
}>

const upgradeModal = ref<ServersUpgradeModalWrapperRef | null>(null)
function openUpgradeModal(serverId: string) {
	upgradeModal.value?.open(serverId)
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.98);
}

.list-enter-active,
.list-leave-active {
	transition: all 200ms ease-in-out;
}

.list-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.list-leave-to {
	opacity: 0;
	transform: translateY(10px);
}

.list-move {
	transition: transform 200ms ease-in-out;
}
</style>
