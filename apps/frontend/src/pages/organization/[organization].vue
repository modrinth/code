<template>
	<div v-if="isLoading" class="flex min-h-[50vh] items-center justify-center">
		<SpinnerIcon class="h-12 w-12 animate-spin text-brand" />
	</div>
	<div
		v-else-if="organization"
		class="new-page sidebar"
		:class="{ 'alt-layout': cosmetics.leftContentLayout || routeHasSettings }"
	>
		<ModalCreation ref="modal_creation" :organization-id="organization.id" />
		<template v-if="routeHasSettings">
			<template v-if="canAccessSettings">
				<div class="normal-page__sidebar">
					<div
						class="bg-surface mb-4 flex flex-col rounded-xl border border-solid border-surface-4 p-4"
					>
						<div class="flex items-center gap-4">
							<Avatar size="sm" :src="organization.icon_url" :raw-src="organization.raw_icon_url" />
							<div class="flex flex-col justify-center gap-1">
								<h2 class="m-0 text-base">
									<nuxt-link :to="`/organization/${organization.slug}`">
										{{ organization.name }}
									</nuxt-link>
								</h2>
								<span>
									{{ formatCompactNumber(acceptedMembers?.length || 0) }}
									member<template v-if="acceptedMembers?.length !== 1">s</template>
								</span>
							</div>
						</div>
					</div>

					<NavStack
						:items="[
							{
								link: `/organization/${organization.slug}/settings`,
								label: 'Overview',
								icon: SettingsIcon,
							},
							{
								link: `/organization/${organization.slug}/settings/members`,
								label: 'Members',
								icon: UsersIcon,
							},
							{
								link: `/organization/${organization.slug}/settings/projects`,
								label: 'Projects',
								icon: BoxIcon,
							},
							{
								link: `/organization/${organization.slug}/settings/analytics`,
								label: 'Analytics',
								icon: ChartIcon,
							},
						]"
					/>
				</div>
				<div class="normal-page__content">
					<NuxtPage />
				</div>
			</template>
		</template>
		<template v-else>
			<div class="normal-page__header py-4">
				<OrganizationPageHeader
					:organization="organization"
					:members-count="acceptedMembers?.length || 0"
					:projects-count="projects?.length || 0"
					:downloads="sumDownloads"
					:can-manage="!!(auth.user && currentMember)"
					@manage-projects="router.push(`/organization/${organization.slug}/settings/projects`)"
					@copy-id="copyId"
					@copy-permalink="copyPermalink"
				/>
			</div>
			<div class="normal-page__sidebar">
				<AdPlaceholder v-if="!auth.user" />

				<SidebarCard title="Members">
					<div class="flex flex-col gap-3 font-semibold">
						<nuxt-link
							v-for="member in acceptedMembers"
							:key="`member-${member?.user?.id}`"
							class="group flex w-fit items-center gap-2 leading-[1.2] text-primary"
							:to="`/user/${member?.user?.username}`"
						>
							<Avatar
								:src="member.user.avatar_url"
								:alt="member.user.username"
								size="32px"
								circle
							/>
							<div class="flex flex-col">
								<span class="flex w-full flex-nowrap items-center gap-1 group-hover:underline">
									<span class="min-w-0 overflow-hidden truncate">{{ member.user.username }}</span>
									<CrownIcon
										v-if="member.is_owner"
										v-tooltip="'Organization owner'"
										class="text-brand-orange"
									/>
								</span>
								<span class="text-sm font-normal text-secondary">
									{{ member?.role ? member.role : 'Member' }}
								</span>
							</div>
						</nuxt-link>
					</div>
				</SidebarCard>
			</div>
			<div class="normal-page__content">
				<div v-if="isInvited" class="universal-card information invited">
					<h2>Invitation to join {{ organization.name }}</h2>
					<p>You have been invited to join {{ organization.name }}.</p>
					<div class="input-group">
						<Button type="colored" color="brand" @click="onAcceptInvite">
							<CheckIcon />
							Accept
						</Button>
						<Button type="colored" color="red" @click="onDeclineInvite">
							<XIcon />
							Decline
						</Button>
					</div>
				</div>
				<NavTabs v-if="navLinks.length > 2" :links="navLinks" replace page-nav />
				<ProjectList
					v-if="projects && projects.length > 0"
					:projects="displayedProjects"
					:show-status="canSeeProjectStatus"
				/>
				<div v-else-if="true" class="error">
					<UpToDate class="icon" />
					<br />
					<span class="preserve-lines text">
						<template v-if="isPermission(currentMember?.permissions, 1 << 4)">
							<IntlFormatted :message-id="messages.noProjectsWithCreatePrompt">
								<template #create-link="{ children }">
									<a class="link" @click="modal_creation?.show()"
										><component :is="() => normalizeChildren(children)"
									/></a>
								</template>
							</IntlFormatted>
						</template>
						<template v-else>{{ formatMessage(messages.noProjects) }}</template>
					</span>
				</div>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BoxIcon,
	ChartIcon,
	CheckIcon,
	CrownIcon,
	SettingsIcon,
	SpinnerIcon,
	UsersIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	catalogProjectTypes,
	commonMessages,
	defineMessages,
	filterProjectsByType,
	injectModrinthClient,
	IntlFormatted,
	NavTabs,
	normalizeChildren,
	parseProjectTypeRouteParam,
	ProjectList,
	SidebarCard,
	useCompactNumber,
	useVIntl,
} from '@modrinth/ui'
import type { Organization, ProjectType } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

import UpToDate from '~/assets/images/illustrations/up_to_date.svg?component'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import ModalCreation from '~/components/ui/create/ProjectCreateModal.vue'
import NavStack from '~/components/ui/NavStack.vue'
import OrganizationPageHeader from '~/components/ui/OrganizationPageHeader.vue'
import { warmProjectCheckCaches } from '~/composables/queries/project'
import { acceptTeamInvite, removeTeamMember } from '~/helpers/teams.js'
import {
	OrganizationContext,
	provideOrganizationContext,
} from '~/providers/organization-context.ts'
import { isPermission } from '~/utils/permissions.ts'
import { projectUserSorting } from '~/utils/projects.ts'

type ProjectV3 = Labrinth.Projects.v3.Project

const vintl = useVIntl()
const { formatMessage } = vintl

const messages = defineMessages({
	noProjects: {
		id: 'organization.projects.none',
		defaultMessage: "This organization doesn't have any projects yet.",
	},
	noProjectsWithCreatePrompt: {
		id: 'organization.projects.none-with-create-prompt',
		defaultMessage:
			"This organization doesn't have any projects yet. Would you like to <create-link>create one</create-link>?",
	},
})

const { formatCompactNumber } = useCompactNumber()

const auth: { user: any } & any = await useAuth()
const cosmetics = useCosmetics()
const route = useNativeRoute()
const router = useRouter()
const tags = useGeneratedState()
const config = useRuntimeConfig()
const modal_creation = useTemplateRef('modal_creation')

const orgId = useRouteId('organization')

if (route.path.includes('settings')) {
	useSeoMeta({
		robots: 'noindex',
	})
}

// hacky way to show the edit button on the corner of the card.
const routeHasSettings = computed(() => route.path.includes('settings'))
useFavicon(() => (routeHasSettings.value ? 'settings' : 'default'))

const client = injectModrinthClient()
const queryClient = useQueryClient()

const {
	data: organization,
	refetch: refreshOrganization,
	error: orgError,
	isPending: organizationIsPending,
} = useQuery({
	queryKey: computed(() => ['organization', orgId]),
	// @ts-expect-error
	queryFn: () => client.labrinth.organizations_v3.get(orgId),
	enabled: !!orgId,
})

watch(
	orgError,
	(error) => {
		if (error) {
			const status = (error as any).statusCode ?? (error as any).status ?? 404
			showError({
				fatal: true,
				statusCode: status,
				message: 'Organization not found',
			})
		}
	},
	{ immediate: true },
)

const {
	data: projects,
	refetch: refreshProjects,
	isFetching: projectsIsFetching,
} = useQuery({
	queryKey: computed(() => ['organization', orgId, 'projects']),
	queryFn: () => client.labrinth.organizations_v3.getProjects(orgId),
	placeholderData: [],
})

watch(
	projects,
	(list) => {
		warmProjectCheckCaches(queryClient, list)
	},
	{ immediate: true },
)

const refresh = async () => {
	await Promise.all([refreshOrganization(), refreshProjects()])
}

// Loading state
const isLoading = computed(() => {
	return organizationIsPending.value || projectsIsFetching.value
})

// Filter accepted, sort by role, then by name and Owner role always goes first
const acceptedMembers = computed(() => {
	const acceptedMembers = organization.value?.members?.filter((x) => x.accepted) ?? []
	const owner = acceptedMembers.find((x) => x.is_owner)
	const rest = acceptedMembers.filter((x) => !x.is_owner) ?? []

	rest.sort((a, b) => {
		if (a.role === b.role) {
			return a.user.username.localeCompare(b.user.username)
		} else {
			return a.role.localeCompare(b.role)
		}
	})

	return owner ? [owner, ...rest] : rest
})

const isInvited = computed(() => {
	return currentMember.value?.accepted === false
})

const projectTypes = computed(() => catalogProjectTypes(projects.value ?? []))

const displayedProjects = computed(() =>
	filterProjectsByType(projects.value ?? [], parseProjectTypeRouteParam(route.params.projectType))
		.slice()
		.sort(projectUserSorting),
)

const sumDownloads = computed(() => {
	let sum = 0

	for (const project of projects.value ?? []) {
		sum += project.downloads
	}

	return sum
})

const onAcceptInvite = useClientTry(async () => {
	await acceptTeamInvite(organization.value?.team_id)
	await refreshOrganization()
})

const onDeclineInvite = useClientTry(async () => {
	await removeTeamMember(organization.value?.team_id, auth.value?.user?.id)
	await refreshOrganization()
})

const organizationContext = new OrganizationContext(
	organization as Ref<Organization | null>,
	projects as Ref<ProjectV3[] | null>,
	auth,
	tags,
	refresh,
)
const { currentMember } = organizationContext

provideOrganizationContext(organizationContext)

const canAccessSettings = computed(() => !!currentMember.value?.accepted)

const authUserId = computed(() => auth.value?.user?.id as string | undefined)
const viewerProjectsQuery = useQuery({
	queryKey: computed(() => ['user', authUserId.value, 'projects']),
	queryFn: () => client.labrinth.users_v3.getProjects(authUserId.value!),
	enabled: computed(() => !!authUserId.value && !!organization.value && !currentMember.value),
	staleTime: 30_000,
})
const viewerMemberProjectIds = computed(
	() => new Set((viewerProjectsQuery.data.value ?? []).map((project) => project.id)),
)

function canSeeProjectStatus(project: ProjectV3) {
	if (currentMember.value) return true
	return viewerMemberProjectIds.value.has(project.id)
}

watch(
	[routeHasSettings, acceptedMembers, currentMember],
	() => {
		if (routeHasSettings.value && acceptedMembers.value.length > 0 && !canAccessSettings.value) {
			showError({
				fatal: true,
				statusCode: 401,
				statusMessage: 'Unauthorized',
			})
		}
	},
	{ flush: 'sync', immediate: true },
)

watch(
	organization,
	(org) => {
		if (org) {
			const title = `${org.name} - Organization`
			const description = `${org.description} - View the organization ${org.name} on Modrinth`
			const canonicalUrl = org ? `https://modrinth.com/organization/${org.id}` : undefined

			useSeoMeta({
				title,
				description,
				ogTitle: title,
				ogDescription: org.description,
				ogImage: org.icon_url ?? 'https://cdn-raw.modrinth.com/placeholder-square.png',
				ogUrl: canonicalUrl,
			})
			useHead({
				link: [
					{
						rel: 'canonical',
						href: canonicalUrl,
					},
				],
			})
		}
	},
	{ immediate: true },
)

const navLinks = computed(() => [
	{
		label: formatMessage(commonMessages.allProjectType),
		href: `/organization/${organization.value?.slug}`,
	},
	...projectTypes.value.map((x) => {
		return {
			label: formatMessage(getProjectTypeMessage(x as ProjectType, true)),
			href: `/organization/${organization.value?.slug}/${x}s`,
		}
	}),
])

async function copyId() {
	await navigator.clipboard.writeText(organization.value?.id ?? '')
}

async function copyPermalink() {
	await navigator.clipboard.writeText(
		`${config.public.siteUrl}/organization/${organization.value?.id}`,
	)
}
</script>

<style scoped lang="scss">
.page-header__settings {
	display: flex;
	flex-direction: row;
	gap: var(--gap-md);
	margin-bottom: var(--gap-md);

	.title-section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: var(--gap-xs);
	}

	.settings-title {
		margin: 0 !important;
		font-size: var(--font-size-md);
	}
}

.page-header__icon {
	margin-block: 0 !important;
}

.universal-card {
	h1 {
		margin-bottom: var(--gap-md);
	}
}

.creator-list {
	display: flex;
	flex-direction: column;
	padding: var(--gap-xl);

	h3 {
		margin: 0 0 var(--gap-sm);
	}

	.creator {
		display: grid;
		gap: var(--gap-xs);
		background-color: var(--color-raised-bg);
		padding: var(--gap-sm);
		margin-left: -0.5rem;
		border-radius: var(--radius-lg);
		grid-template:
			'avatar name' auto
			'avatar role' auto
			/ auto 1fr;

		p {
			margin: 0;
		}

		.name {
			grid-area: name;
			align-self: flex-end;
			margin-left: var(--gap-xs);
			font-weight: bold;

			display: flex;
			align-items: center;
			gap: 0.25rem;

			svg {
				color: var(--color-orange);
			}
		}

		.role {
			grid-area: role;
			align-self: flex-start;
			margin-left: var(--gap-xs);
		}

		.avatar {
			grid-area: avatar;
		}
	}
}

.secondary-stat {
	align-items: center;
	display: flex;
	margin-bottom: 0.8rem;
}

.secondary-stat__icon {
	height: 1rem;
	width: 1rem;
}

.secondary-stat__text {
	margin-left: 0.4rem;
	display: flex;
	align-items: center;
	gap: 0.5rem;
}

.title {
	margin: var(--gap-md) 0 var(--spacing-card-xs) 0;
	font-size: var(--font-size-xl);
	color: var(--color-text-dark);
}

.organization-label {
	font-weight: 500;
	display: flex;
	align-items: center;
	gap: 0.25rem;
}

.organization-description {
	margin-top: var(--spacing-card-sm);
	margin-bottom: 0;
}

.title-and-link {
	display: flex;
	justify-content: space-between;
	align-items: center;

	h3 {
		margin: 0;
	}

	a {
		display: flex;
		align-items: center;
		gap: var(--gap-xs);
		color: var(--color-blue);
	}
}

.project-overview {
	gap: var(--gap-md);
	padding: var(--gap-xl);

	.project-card {
		padding: 0;
		border-radius: 0;
		background-color: transparent;
		box-shadow: none;

		:deep(.title) {
			font-size: var(--font-size-nm) !important;
		}
	}
}

.popout-heading {
	padding: var(--gap-sm) var(--gap-md);
	margin: 0;
	font-size: var(--font-size-md);
	color: var(--color-text);
}

.popout-checkbox {
	padding: var(--gap-sm) var(--gap-md);
}

.new-page {
	column-gap: 1.5rem;
}
</style>
