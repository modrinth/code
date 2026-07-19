import type { Labrinth } from '@modrinth/api-client'
import { ModrinthApiError } from '@modrinth/api-client'
import {
	BookOpenIcon,
	TagCategoryRefreshCcwIcon,
	TagCategoryWandSparklesIcon,
	UserPlusIcon,
} from '@modrinth/assets'
import { Alert, injectModrinthClient, injectProjectPageContext } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import type { NodeState } from '../../types/node'
import { button, fix, group, option, stage, text, toggle } from '../../types/node'

//TODO: make this not a copy of frontend/src/utils/slugs.generateUrlSlug
// (as in move the other one so we can use it here)
function generateUrlSlug(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replaceAll(' ', '-')
		.replaceAll(/[^a-zA-Z0-9._-]/g, '')
		.replaceAll(/--+/gm, '-')
}

function hasCustomSlug(project: Labrinth.Projects.v3.Project) {
	return generateUrlSlug(project.name) !== project.slug
}

export default function () {
	const { projectV3: project } = injectProjectPageContext()
	const client = injectModrinthClient()

	type AutoSlugStatus = 'loading' | 'available' | 'unavailable'
	const autoSlugStatus = ref<AutoSlugStatus>('loading')
	const resolvedAutoSlug = ref<string | null>(null)
	const takenBy = ref<Labrinth.Projects.v3.Project | null>(null)
	const autoSlugTakenBy = ref<Labrinth.Projects.v3.Project | null>(null)
	const ownerUsername = ref<string | null>(null)

	function currentSlug(state: Record<string, NodeState>) {
		return (state['correct-slug'] as string | undefined) ?? project.value.slug
	}

	type SlugValidation = 'checking' | 'available' | 'unchanged' | 'taken' | null
	const slugValidation = ref<SlugValidation>(null)
	let slugDebounceTimer: ReturnType<typeof setTimeout> | undefined

	async function checkSlugTaken(slug: string): Promise<Labrinth.Projects.v3.Project | null> {
		try {
			return await client.labrinth.projects_v3.get(slug)
		} catch (e) {
			if (e instanceof ModrinthApiError && e.statusCode === 404) return null
			throw e
		}
	}

	const SlugStatus = () => {
		const v = slugValidation.value
		if (v === null) return null
		if (v === 'checking')
			return (
				<Alert type="checking" class="w-full">
					Checking availability…
				</Alert>
			)
		if (v === 'unchanged')
			return (
				<Alert type="warning" class="w-full">
					Already the current slug
				</Alert>
			)
		if (v === 'available')
			return (
				<Alert type="success" class="w-full">
					Slug is available
				</Alert>
			)
		const by = takenBy.value
		return (
			<Alert type="error" class="w-full">
				Slug taken
				{by ? (
					<>
						{' by '}
						<a href={`/project/${by.slug}`} target="_blank" class="underline">
							{by.name}
						</a>
						{` (${by.status})`}
					</>
				) : null}
			</Alert>
		)
	}

	watch(
		[() => project.value.name, () => project.value.slug],
		async ([name]) => {
			autoSlugStatus.value = 'loading'
			resolvedAutoSlug.value = null
			takenBy.value = null
			autoSlugTakenBy.value = null

			if (!hasCustomSlug(project.value)) {
				autoSlugStatus.value = 'available'
				resolvedAutoSlug.value = generateUrlSlug(name)
				return
			}

			const autoSlug = generateUrlSlug(name)

			try {
				const [conflict, members] = await Promise.all([
					checkSlugTaken(autoSlug),
					ownerUsername.value === null
						? client.labrinth.projects_v3.getMembers(project.value.id)
						: null,
				])

				if (members) {
					const owner = members.find((m) => m.is_owner) ?? members[0]
					ownerUsername.value = owner
						? owner.user.username.toLowerCase().replaceAll(/[^a-z0-9-]/g, '') || null
						: null
				}

				if (!conflict) {
					resolvedAutoSlug.value = autoSlug
					autoSlugStatus.value = 'available'
					return
				}

				autoSlugTakenBy.value = conflict

				if (ownerUsername.value) {
					const withUser = `${autoSlug}-${ownerUsername.value}`
					const conflict2 = await checkSlugTaken(withUser)
					if (!conflict2) {
						resolvedAutoSlug.value = withUser
						autoSlugStatus.value = 'available'
						return
					}
				}
			} catch {}

			autoSlugStatus.value = 'unavailable'
		},
		{ immediate: true },
	)

	return stage('title-slug', 'Title & Slug')
		.hint('Are the Name and URL accurate and appropriate?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
		)
		.icon(BookOpenIcon)
		.children(
			() => {
				if (!hasCustomSlug(project.value)) {
					return (
						<div class="markdown-body w-full">
							<strong>Title:</strong> {project.value.name}
						</div>
					)
				}
				const by = autoSlugTakenBy.value
				return (
					<div class="markdown-body w-full">
						<strong>Title:</strong> {project.value.name}
						<br/>
						<strong>Slug:</strong>{' '}
						<code>{project.value.slug}</code>
						<br/>
						<strong>Slug Taken:</strong>{' '}
						{autoSlugStatus.value === 'loading' ? (
							'...'
						) : by ? (
							<>
								<a href={`/project/${by.slug}`} target="_blank" class="underline">
									{by.name}
								</a>
								{` (${by.status})`}
							</>
						) : (
							'No'
						)}
					</div>
				)
			},

			group('title')
				.title('Title Issues?')
				.children(
					toggle('useless-info', 'Contains Useless Info')
						.suggestedStatus('flagged')
						.severity('low')
						.message(),

					toggle('minecraft-branding', 'Minecraft Title')
						.suggestedStatus('flagged')
						.severity('medium')
						.message(),

					toggle('similarities', 'Title Similarities')
						.suggestedStatus('flagged')
						.severity('medium')
						.message()
						.children(
							group()
								.title('Similarities Additional Info')
								.multiSelect('options')
								.children(
									option('modpack', 'Modpack Named After Mod')
										.shown(computed(() => project.value.project_types.includes('modpack')))
										.message(),

									option('fork', 'Forked Project')
										.shown(computed(() => !project.value?.minecraft_server))
										.message(),
								),
						)
						.collect(),
				),

			group('slug')
				.title('Slug Issues')
				.shown(computed(() => hasCustomSlug(project.value)))
				.children(
					group()
						.multiSelect('issues')
						.children(
							toggle('misused', 'Misused')
								.children(
									group()
										.title('Correct Slug')
										.children(
											text('correct-slug')
												.initial(project.value.slug)
												.onChange((value, { override }) => {
													if (!value) return override(project.value.slug ?? '')
													clearTimeout(slugDebounceTimer)
													if (value === project.value.slug) {
														slugValidation.value = 'unchanged'
														return
													}
													slugValidation.value = 'checking'
													slugDebounceTimer = setTimeout(async () => {
														const conflict = await checkSlugTaken(value).catch(() => null)
														if (conflict !== null && conflict.id !== project.value.id) {
															takenBy.value = conflict
															slugValidation.value = 'taken'
														} else {
															takenBy.value = null
															slugValidation.value = 'available'
														}
													}, 400)
												}),

											button()
												.icon(TagCategoryWandSparklesIcon)
												.tooltip(computed(() => resolvedAutoSlug.value ?? ''))
												.enabled(
													(state) =>
														autoSlugStatus.value === 'available' &&
														resolvedAutoSlug.value !== null &&
														resolvedAutoSlug.value !== currentSlug(state),
												)
												.onClick((state) => {
													if (resolvedAutoSlug.value) state['correct-slug'] = resolvedAutoSlug.value
												}),

											button()
												.icon(UserPlusIcon)
												.tooltip((state) => {
													const current = currentSlug(state)
													if (!ownerUsername.value || current?.includes(ownerUsername.value))
														return current ?? ''
													return `${current}-${ownerUsername.value}`
												})
												.enabled(
													(state) =>
														ownerUsername.value !== null &&
														!currentSlug(state)?.includes(ownerUsername.value),
												)
												.onClick((state) => {
													state['correct-slug'] = `${currentSlug(state)}-${ownerUsername.value}`
												}),

											button()
												.icon(TagCategoryRefreshCcwIcon)
												.tooltip(computed(() => project.value.slug ?? ''))
												.enabled((state) => currentSlug(state) !== project.value.slug)
												.onClick((state) => {
													state['correct-slug'] = undefined
												}),

											SlugStatus,
										),
								)
								.message(undefined, (state) => ({
									SUGGESTED_SLUG: state['correct-slug'],
								}))
								.fix(
									fix().project((patch, state) => {
										const slug = state['correct-slug'] as string
										if (!slug) return
										patch.slug = slug
									}),
								),
						),
				),
		)
}
