<template>
	<div>
		<h2 class="m-0 mb-4 text-2xl font-semibold">Server lookup</h2>
		<form class="card flex flex-col gap-3" @submit.prevent="lookupServer">
			<div class="flex flex-col gap-2">
				<label for="server-subdomain">
					<span class="text-lg font-semibold text-contrast">
						Server subdomain
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<Input
					id="server-subdomain"
					v-model="subdomainInput"
					:icon="ServerIcon"
					:error="submitted && !!validationError"
					placeholder="example or example.modrinth.gg"
					autocomplete="off"
					autocapitalize="none"
					:spellcheck="false"
				/>
				<span v-if="submitted && validationError" class="text-sm font-medium text-red">
					{{ validationError }}
				</span>
			</div>

			<div>
				<Button type="colored" color="brand" native-type="submit" :loading="isLoading">
					<SearchIcon aria-hidden="true" />
					Look up server
				</Button>
			</div>

			<Admonition v-if="lookupError" type="critical" header="Lookup failed">
				{{ lookupError }}
			</Admonition>
		</form>
	</div>
</template>

<script setup lang="ts">
import { ModrinthApiError } from '@modrinth/api-client'
import { SearchIcon, ServerIcon } from '@modrinth/assets'
import { Admonition, Button, injectModrinthClient, Input } from '@modrinth/ui'

const client = injectModrinthClient()

const subdomainInput = ref('')
const submitted = ref(false)
const isLoading = ref(false)
const lookupError = ref('')

const normalizedSubdomain = computed(() =>
	subdomainInput.value
		.trim()
		.replace(/\.modrinth\.gg$/i, '')
		.toLowerCase(),
)
const validationError = computed(() => {
	if (!normalizedSubdomain.value) {
		return 'Enter a server subdomain.'
	}

	if (normalizedSubdomain.value.length > 63) {
		return 'Subdomain must be 63 characters or fewer.'
	}

	if (!/^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/.test(normalizedSubdomain.value)) {
		return 'Use only letters, numbers, and hyphens, without a leading or trailing hyphen.'
	}

	return undefined
})

watch(subdomainInput, (value) => {
	const withoutDomain = value.replace(/\.modrinth\.gg$/i, '')
	if (withoutDomain !== value) {
		subdomainInput.value = withoutDomain
	}
	lookupError.value = ''
})

async function lookupServer() {
	if (isLoading.value) {
		return
	}

	submitted.value = true
	lookupError.value = ''

	if (validationError.value) {
		return
	}

	subdomainInput.value = normalizedSubdomain.value
	isLoading.value = true

	try {
		const server = await client.archon.servers_internal.getBySubdomain(normalizedSubdomain.value)
		await navigateTo(`/hosting/manage/${server.id}`)
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 404) {
			lookupError.value = `No server was found for ${normalizedSubdomain.value}.modrinth.gg.`
		} else {
			lookupError.value = 'The server could not be looked up. Try again.'
		}
	} finally {
		isLoading.value = false
	}
}
</script>
