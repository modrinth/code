<template>
	<Teleport v-if="flags.projectBackground" to="#fixed-background-teleport">
		<ProjectBackgroundGradient :project="project" />
	</Teleport>
	<div v-if="route.name.startsWith('type-id-settings')" class="normal-page no-sidebar">
		<div class="normal-page__header">
			<div
				class="mb-4 flex flex-wrap items-center gap-x-2 gap-y-3 border-0 border-b-[1px] border-solid border-divider pb-4 text-lg font-semibold"
			>
				<nuxt-link
					:to="`/${project.project_type}/${project.slug ? project.slug : project.id}`"
					class="flex items-center gap-2 hover:underline hover:brightness-[--hover-brightness]"
				>
					<Avatar :src="project.icon_url" size="32px" />
					{{ project.title }}
				</nuxt-link>
				<ChevronRightIcon />
				<span class="flex grow font-extrabold text-contrast">{{
					formatMessage(messages.settingsTitle)
				}}</span>
				<div class="flex gap-2">
					<ButtonStyled>
						<nuxt-link to="/dashboard/projects"
							><ListIcon /> {{ formatMessage(messages.visitProjectsDashboard) }}
						</nuxt-link>
					</ButtonStyled>
				</div>
			</div>
			<ProjectMemberHeader
				v-if="currentMember && false"
				:project="project"
				:versions="versions"
				:current-member="currentMember"
				:is-settings="route.name.startsWith('type-id-settings')"
				:set-processing="setProcessing"
				:all-members="allMembers"
				:update-members="updateMembers"
				:auth="auth"
				:tags="tags"
			/>
		</div>
		<div class="normal-page__content">
			<NuxtPage
				v-model:project="project"
				v-model:project-v3="projectV3"
				v-model:versions="versions"
				v-model:members="members"
				v-model:all-members="allMembers"
				v-model:dependencies="dependencies"
				v-model:organization="organization"
				:current-member="currentMember"
				:patch-project="patchProject"
				:patch-icon="patchIcon"
				:reset-project="resetProject"
				:reset-versions="resetVersions"
				:reset-organization="resetOrganization"
				:reset-members="resetMembers"
				:route="route"
			/>
		</div>
	</div>

	<div v-else class="experimental-styles-within">
		<NewModal ref="settingsModal">
			<template #title>
				<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
				<span class="text-lg font-extrabold text-contrast">
					{{ formatMessage(messages.settingsTitle) }}
				</span>
			</template>
		</NewModal>
		<NewModal
			ref="modalLicense"
			:header="project.license.name ? project.license.name : formatMessage(messages.licenseTitle)"
		>
			<template #title>
				<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" no-shadow />
				<span class="text-lg font-extrabold text-contrast">
					{{ project.license.name ? project.license.name : formatMessage(messages.licenseTitle) }}
				</span>
			</template>
			<div
				class="markdown-body"
				v-html="
					renderString(licenseText).isEmpty
						? formatMessage(messages.loadingLicenseText)
						: renderString(licenseText)
				"
			/>
		</NewModal>
		<div
			class="over-the-top-download-animation"
			:class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
		>
			<div>
				<div
					class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
				></div>

				<div
					class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
				></div>

				<div
					class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
				>
					<DownloadIcon class="h-20 w-20 text-contrast" />
				</div>
			</div>
		</div>
		<NewModal
			ref="downloadModal"
			:on-show="
				() => {
					navigateTo({ query: route.query, hash: '#download' })
				}
			"
			:on-hide="
				() => {
					navigateTo({ query: route.query, hash: '' })
				}
			"
		>
			<template #title>
				<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
				<div class="truncate text-lg font-extrabold text-contrast">
					{{ formatMessage(messages.downloadTitle, { title: project.title }) }}
				</div>
			</template>
			<template #default>
				<div class="mx-auto flex max-w-[40rem] flex-col gap-4 md:w-[30rem]">
					<div
						v-if="
							project.project_type !== 'plugin' ||
							project.loaders.some((x) => !tags.loaderData.allPluginLoaders.includes(x))
						"
						class="modrinth-app-section contents"
					>
						<div class="mx-auto flex w-fit flex-col">
							<ButtonStyled color="brand">
								<a
									class="w-fit"
									:href="`modrinth://mod/${project.slug}`"
									@click="() => installWithApp()"
								>
									<ModrinthIcon aria-hidden="true" />
									{{ formatMessage(messages.installWithModrinthApp) }}
									<ExternalIcon aria-hidden="true" />
								</a>
							</ButtonStyled>
							<Accordion ref="getModrinthAppAccordion">
								<nuxt-link
									class="mt-2 flex justify-center text-brand-blue hover:underline"
									to="/app"
								>
									{{ formatMessage(messages.dontHaveModrinthApp) }}
								</nuxt-link>
							</Accordion>
						</div>

						<div class="flex items-center gap-4 px-4">
							<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
							<span class="flex-shrink-0 text-sm font-semibold text-secondary">
								{{ formatMessage(commonMessages.orLabel) }}
							</span>
							<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
						</div>
					</div>

					<div class="mx-auto flex w-fit flex-col gap-2">
						<ButtonStyled v-if="project.game_versions.length === 1">
							<div class="disabled button-like">
								<GameIcon aria-hidden="true" />
								{{
									currentGameVersion
										? formatMessage(messages.gameVersionLabel, { version: currentGameVersion })
										: formatMessage(messages.gameVersionError)
								}}
								<InfoIcon
									v-tooltip="
										formatMessage(messages.gameVersionTooltip, {
											title: project.title,
											version: currentGameVersion,
										})
									"
									class="ml-auto size-5"
								/>
							</div>
						</ButtonStyled>
						<Accordion
							v-else
							ref="gameVersionAccordion"
							class="accordion-with-bg"
							@on-open="
								() => {
									if (platformAccordion) {
										platformAccordion.close()
									}
								}
							"
						>
							<template #title>
								<GameIcon aria-hidden="true" />
								{{
									currentGameVersion
										? formatMessage(messages.gameVersionLabel, { version: currentGameVersion })
										: formatMessage(messages.selectGameVersion)
								}}
							</template>
							<div class="iconified-input mb-2 flex w-full">
								<label for="game-versions-filtering" hidden>{{
									formatMessage(messages.searchGameVersionsLabel)
								}}</label>
								<SearchIcon aria-hidden="true" />
								<input
									id="game-versions-filtering"
									ref="gameVersionFilterInput"
									v-model="versionFilter"
									type="search"
									autocomplete="off"
									:placeholder="formatMessage(messages.searchGameVersions)"
								/>
							</div>
							<ScrollablePanel :class="project.game_versions.length > 4 ? 'h-[15rem]' : ''">
								<ButtonStyled
									v-for="gameVersion in project.game_versions
										.filter(
											(x) =>
												(versionFilter && x.includes(versionFilter)) ||
												(!versionFilter && (showAllVersions || isReleaseGameVersion(x))),
										)
										.slice()
										.reverse()"
									:key="gameVersion"
									:color="currentGameVersion === gameVersion ? 'brand' : 'standard'"
								>
									<button
										v-tooltip="
											!possibleGameVersions.includes(gameVersion)
												? formatMessage(messages.gameVersionUnsupportedTooltip, {
														title: project.title,
														gameVersion: gameVersion,
														platform: formatCategory(currentPlatform),
													})
												: null
										"
										:class="{
											'looks-disabled !text-brand-red': !possibleGameVersions.includes(gameVersion),
										}"
										@click="
											() => {
												userSelectedGameVersion = gameVersion
												gameVersionAccordion.close()
												if (!currentPlatform && platformAccordion) {
													platformAccordion.open()
												}

												navigateTo({
													query: {
														...route.query,
														...(userSelectedGameVersion && {
															version: userSelectedGameVersion,
														}),
														...(userSelectedPlatform && {
															loader: userSelectedPlatform,
														}),
													},
													hash: route.hash,
												})
											}
										"
									>
										{{ gameVersion }}
										<CheckIcon v-if="userSelectedGameVersion === gameVersion" />
									</button>
								</ButtonStyled>
							</ScrollablePanel>
							<Checkbox
								v-if="showVersionsCheckbox"
								v-model="showAllVersions"
								class="mx-1"
								:label="formatMessage(messages.showAllVersions)"
								:disabled="!!versionFilter"
							/>
						</Accordion>
						<ButtonStyled
							v-if="project.loaders.length === 1 && project.project_type !== 'resourcepack'"
						>
							<div class="disabled button-like">
								<WrenchIcon aria-hidden="true" />
								{{
									currentPlatform
										? formatMessage(messages.platformLabel, {
												platform: formatCategory(currentPlatform),
											})
										: formatMessage(messages.platformError)
								}}
								<InfoIcon
									v-tooltip="
										formatMessage(messages.platformTooltip, {
											title: project.title,
											platform: formatCategory(currentPlatform),
										})
									"
									class="ml-auto size-5"
								/>
							</div>
						</ButtonStyled>
						<Accordion
							v-else-if="project.project_type !== 'resourcepack'"
							ref="platformAccordion"
							class="accordion-with-bg"
							@on-open="
								() => {
									if (gameVersionAccordion) {
										gameVersionAccordion.close()
									}
								}
							"
						>
							<template #title>
								<WrenchIcon aria-hidden="true" />
								{{
									currentPlatform
										? formatMessage(messages.platformLabel, {
												platform: formatCategory(currentPlatform),
											})
										: formatMessage(messages.selectPlatform)
								}}
							</template>
							<ScrollablePanel :class="project.loaders.length > 4 ? 'h-[15rem]' : ''">
								<ButtonStyled
									v-for="platform in project.loaders.slice().reverse()"
									:key="platform"
									:color="currentPlatform === platform ? 'brand' : 'standard'"
								>
									<button
										v-tooltip="
											!possiblePlatforms.includes(platform)
												? formatMessage(messages.platformUnsupportedTooltip, {
														title: project.title,
														platform: formatCategory(platform),
														gameVersion: currentGameVersion,
													})
												: null
										"
										:class="{
											'looks-disabled !text-brand-red': !possiblePlatforms.includes(platform),
										}"
										@click="
											() => {
												userSelectedPlatform = platform

												platformAccordion.close()
												if (!currentGameVersion && gameVersionAccordion) {
													gameVersionAccordion.open()
												}

												navigateTo({
													query: {
														...route.query,
														...(userSelectedGameVersion && {
															version: userSelectedGameVersion,
														}),
														...(userSelectedPlatform && {
															loader: userSelectedPlatform,
														}),
													},
													hash: route.hash,
												})
											}
										"
									>
										{{ formatCategory(platform) }}
										<CheckIcon v-if="userSelectedPlatform === platform" />
									</button>
								</ButtonStyled>
							</ScrollablePanel>
						</Accordion>
					</div>
					<AutomaticAccordion div class="flex flex-col gap-2">
						<VersionSummary
							v-if="filteredRelease"
							:version="filteredRelease"
							@on-download="onDownload"
							@on-navigate="downloadModal.hide"
						/>
						<VersionSummary
							v-if="filteredBeta"
							:version="filteredBeta"
							@on-download="onDownload"
							@on-navigate="downloadModal.hide"
						/>
						<VersionSummary
							v-if="filteredAlpha"
							:version="filteredAlpha"
							@on-download="onDownload"
							@on-navigate="downloadModal.hide"
						/>
						<p
							v-if="
								currentPlatform &&
								currentGameVersion &&
								!filteredRelease &&
								!filteredBeta &&
								!filteredAlpha
							"
						>
							{{
								formatMessage(messages.noVersionsAvailable, {
									gameVersion: currentGameVersion,
									platform: formatCategory(currentPlatform),
								})
							}}
						</p>
					</AutomaticAccordion>
					<ServersPromo
						v-if="flags.showProjectPageDownloadModalServersPromo"
						:link="`/hosting#plan`"
						@close="
							() => {
								flags.showProjectPageDownloadModalServersPromo = false
								saveFeatureFlags()
							}
						"
					/>
				</div>
			</template>
		</NewModal>
		<CollectionCreateModal ref="modal_collection" :project-ids="[project.id]" />
		<div
			class="new-page sidebar"
			:class="{
				'alt-layout': cosmetics.leftContentLayout,
				'checklist-open':
					showModerationChecklist &&
					!collapsedModerationChecklist &&
					!flags.alwaysShowChecklistAsPopup,
				'checklist-collapsed':
					showModerationChecklist &&
					collapsedModerationChecklist &&
					!flags.alwaysShowChecklistAsPopup,
			}"
		>
			<div class="normal-page__header relative my-4">
				<ProjectHeader :project="project" :member="!!currentMember">
					<template #actions>
						<ButtonStyled v-if="auth.user && currentMember" size="large" color="brand">
							<nuxt-link
								:to="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings`"
								class="!font-bold"
							>
								<SettingsIcon aria-hidden="true" />
								Edit project
							</nuxt-link>
						</ButtonStyled>

						<div class="hidden sm:contents">
							<ButtonStyled
								v-tooltip="
									auth.user && currentMember ? formatMessage(commonMessages.downloadButton) : ''
								"
								size="large"
								:color="
									(auth.user && currentMember) || route.name === 'type-id-version-version'
										? `standard`
										: `brand`
								"
								:circular="auth.user && currentMember"
							>
								<button @click="(event) => downloadModal.show(event)">
									<DownloadIcon aria-hidden="true" />
									{{
										auth.user && currentMember ? '' : formatMessage(commonMessages.downloadButton)
									}}
								</button>
							</ButtonStyled>
						</div>

						<div class="contents sm:hidden">
							<ButtonStyled
								size="large"
								circular
								:color="route.name === 'type-id-version-version' ? `standard` : `brand`"
							>
								<button
									:aria-label="formatMessage(commonMessages.downloadButton)"
									class="flex sm:hidden"
									@click="(event) => downloadModal.show(event)"
								>
									<DownloadIcon aria-hidden="true" />
								</button>
							</ButtonStyled>
						</div>
						<Tooltip
							v-if="canCreateServerFrom && flags.showProjectPageQuickServerButton"
							theme="dismissable-prompt"
							:triggers="[]"
							:shown="flags.showProjectPageCreateServersTooltip"
							:auto-hide="false"
							placement="bottom-start"
						>
							<ButtonStyled size="large" circular>
								<nuxt-link
									v-tooltip="formatMessage(messages.createServerTooltip)"
									:to="`/hosting?project=${project.id}#plan`"
									@click="
										() => {
											flags.showProjectPageCreateServersTooltip = false
											saveFeatureFlags()
										}
									"
								>
									<ServerPlusIcon aria-hidden="true" />
								</nuxt-link>
							</ButtonStyled>
							<template #popper>
								<div class="experimental-styles-within grid grid-cols-[min-content] gap-1">
									<div class="flex min-w-60 items-center justify-between gap-4">
										<h3
											class="m-0 flex items-center gap-2 whitespace-nowrap text-base font-bold text-contrast"
										>
											{{ formatMessage(messages.serversPromoTitle) }}
											<TagItem
												:style="{
													'--_color': 'var(--color-brand)',
													'--_bg-color': 'var(--color-brand-highlight)',
												}"
												>{{ formatMessage(commonMessages.newBadge) }}</TagItem
											>
										</h3>
										<ButtonStyled size="small" circular>
											<button
												v-tooltip="formatMessage(messages.dontShowAgain)"
												@click="
													() => {
														flags.showProjectPageCreateServersTooltip = false
														saveFeatureFlags()
													}
												"
											>
												<XIcon aria-hidden="true" />
											</button>
										</ButtonStyled>
									</div>

									<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
										{{ formatMessage(messages.serversPromoDescription) }}
									</p>

									<p class="m-0 text-wrap text-sm font-bold text-primary">
										<IntlFormatted
											:message-id="messages.serversPromoPricing"
											:values="{
												price: formatPrice(locale, 500, 'USD', true),
											}"
										>
											<template #small="{ children }">
												<span class="text-xs">
													<component :is="() => children" />
												</span>
											</template>
										</IntlFormatted>
									</p>
								</div>
							</template>
						</Tooltip>
						<ButtonStyled size="large" circular>
							<ClientOnly>
								<button
									v-if="auth.user"
									v-tooltip="
										following
											? formatMessage(commonMessages.unfollowButton)
											: formatMessage(commonMessages.followButton)
									"
									:aria-label="
										following
											? formatMessage(commonMessages.unfollowButton)
											: formatMessage(commonMessages.followButton)
									"
									@click="userFollowProject(project)"
								>
									<HeartIcon :fill="following ? 'currentColor' : 'none'" aria-hidden="true" />
								</button>
								<nuxt-link
									v-else
									v-tooltip="formatMessage(commonMessages.followButton)"
									to="/auth/sign-in"
									:aria-label="formatMessage(commonMessages.followButton)"
								>
									<HeartIcon aria-hidden="true" />
								</nuxt-link>
								<template #fallback>
									<nuxt-link
										v-tooltip="formatMessage(commonMessages.followButton)"
										to="/auth/sign-in"
										:aria-label="formatMessage(commonMessages.followButton)"
									>
										<HeartIcon aria-hidden="true" />
									</nuxt-link>
								</template>
							</ClientOnly>
						</ButtonStyled>
						<ButtonStyled size="large" circular>
							<PopoutMenu
								v-if="auth.user"
								:tooltip="
									collections.some((x) => x.projects.includes(project.id))
										? formatMessage(commonMessages.savedLabel)
										: formatMessage(commonMessages.saveButton)
								"
								from="top-right"
								:aria-label="formatMessage(commonMessages.saveButton)"
								:dropdown-id="`${baseId}-save`"
							>
								<BookmarkIcon
									aria-hidden="true"
									:fill="
										collections.some((x) => x.projects.includes(project.id))
											? 'currentColor'
											: 'none'
									"
								/>
								<template #menu>
									<input
										v-model="displayCollectionsSearch"
										type="text"
										:placeholder="formatMessage(commonMessages.searchPlaceholder)"
										class="search-input menu-search"
									/>
									<div v-if="collections.length > 0" class="collections-list text-primary">
										<Checkbox
											v-for="option in collections
												.slice()
												.sort((a, b) => a.name.localeCompare(b.name))"
											:key="option.id"
											:model-value="option.projects.includes(project.id)"
											class="popout-checkbox"
											@update:model-value="() => onUserCollectProject(option, project.id)"
										>
											{{ option.name }}
										</Checkbox>
									</div>

									<div v-else class="menu-text">
										<p class="popout-text">{{ formatMessage(messages.noCollectionsFound) }}</p>
									</div>
									<button
										class="btn collection-button"
										@click="(event) => $refs.modal_collection.show(event)"
									>
										<PlusIcon aria-hidden="true" />
										{{ formatMessage(messages.createNewCollection) }}
									</button>
								</template>
							</PopoutMenu>
							<nuxt-link v-else v-tooltip="'Save'" to="/auth/sign-in" aria-label="Save">
								<BookmarkIcon aria-hidden="true" />
							</nuxt-link>
						</ButtonStyled>

						<ButtonStyled size="large" circular type="transparent">
							<OverflowMenu
								:tooltip="formatMessage(commonMessages.moreOptionsButton)"
								:options="[
									{
										id: 'analytics',
										link: `/${project.project_type}/${project.slug ? project.slug : project.id}/settings/analytics`,
										hoverOnly: true,
										shown: auth.user && !!currentMember,
									},
									{
										divider: true,
										shown: auth.user && !!currentMember,
									},
									{
										id: 'moderation-checklist',
										action: () => {
											moderationStore.setSingleProject(project.id)
											showModerationChecklist = true
										},
										color: 'orange',
										hoverOnly: true,
										shown:
											auth.user &&
											tags.staffRoles.includes(auth.user.role) &&
											!showModerationChecklist,
									},
									{
										divider: true,
										shown:
											auth.user &&
											tags.staffRoles.includes(auth.user.role) &&
											!showModerationChecklist,
									},
									{
										id: 'report',
										action: () =>
											auth.user ? reportProject(project.id) : navigateTo('/auth/sign-in'),
										color: 'red',
										hoverOnly: true,
										shown: !isMember,
									},
									{ id: 'copy-id', action: () => copyId() },
									{ id: 'copy-permalink', action: () => copyPermalink() },
								]"
								:aria-label="formatMessage(commonMessages.moreOptionsButton)"
								:dropdown-id="`${baseId}-more-options`"
							>
								<MoreVerticalIcon aria-hidden="true" />
								<template #analytics>
									<ChartIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.analyticsButton) }}
								</template>
								<template #moderation-checklist>
									<ScaleIcon aria-hidden="true" /> {{ formatMessage(messages.reviewProject) }}
								</template>
								<template #report>
									<ReportIcon aria-hidden="true" /> {{ formatMessage(commonMessages.reportButton) }}
								</template>
								<template #copy-id>
									<ClipboardCopyIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.copyIdButton) }}
								</template>
								<template #copy-permalink>
									<ClipboardCopyIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.copyPermalinkButton) }}
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</template>
				</ProjectHeader>
				<ProjectMemberHeader
					v-if="currentMember"
					:project="project"
					:versions="versions"
					:current-member="currentMember"
					:is-settings="route.name.startsWith('type-id-settings')"
					:route-name="route.name"
					:set-processing="setProcessing"
					:collapsed="collapsedChecklist"
					:toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
					:all-members="allMembers"
					:update-members="updateMembers"
					:auth="auth"
					:tags="tags"
				/>
				<Admonition
					v-if="
						currentMember &&
						projectV3.side_types_migration_review_status === 'pending' &&
						projectV3.environment?.length === 1 &&
						projectV3.environment[0] !== 'unknown'
					"
					type="warning"
					:header="
						formatMessage(
							hasEditDetailsPermission
								? messages.environmentMigrationTitle
								: messages.environmentMigrationNoPermissionTitle,
						)
					"
					class="mt-3"
				>
					{{
						formatMessage(
							hasEditDetailsPermission
								? messages.environmentMigrationMessage
								: messages.environmentMigrationNoPermissionMessage,
						)
					}}
					<nuxt-link
						to="/news/article/new-environments"
						target="_blank"
						class="mt-1 block w-fit font-semibold text-orange hover:underline"
					>
						{{ formatMessage(messages.environmentMigrationLink) }}
					</nuxt-link>
					<ButtonStyled v-if="hasEditDetailsPermission" color="orange">
						<nuxt-link :to="`/project/${project.id}/settings/environment`" class="mt-3 w-fit">
							<SettingsIcon /> {{ formatMessage(messages.reviewEnvironmentSettings) }}
						</nuxt-link>
					</ButtonStyled>
				</Admonition>
				<MessageBanner v-if="project.status === 'archived'" message-type="warning" class="my-4">
					{{ formatMessage(messages.archivedMessage, { title: project.title }) }}
				</MessageBanner>
			</div>

			<div class="normal-page__sidebar">
				<ProjectSidebarCompatibility
					:project="project"
					:tags="tags"
					:v3-metadata="projectV3"
					class="card flex-card experimental-styles-within"
				/>
				<AdPlaceholder v-if="!auth.user && tags.approvedStatuses.includes(project.status)" />
				<ProjectSidebarLinks
					:project="project"
					:link-target="$external()"
					class="card flex-card experimental-styles-within"
				/>
				<ProjectSidebarCreators
					:organization="organization"
					:members="members"
					:org-link="(slug) => `/organization/${slug}`"
					:user-link="(username) => `/user/${username}`"
					class="card flex-card experimental-styles-within"
				/>
				<!-- TODO: Finish license modal and enable -->
				<ProjectSidebarDetails
					v-if="false"
					:project="project"
					:has-versions="versions.length > 0"
					:link-target="$external()"
					class="card flex-card experimental-styles-within"
				/>
				<div class="card flex-card experimental-styles-within">
					<h2>{{ formatMessage(detailsMessages.title) }}</h2>

					<div class="details-list">
						<div class="details-list__item">
							<BookTextIcon aria-hidden="true" />
							<div>
								{{ formatMessage(messages.licensedLabel) }}
								<a
									v-if="project.license.url"
									class="text-link hover:underline"
									:href="project.license.url"
									:target="$external()"
									rel="noopener nofollow ugc"
								>
									{{ licenseIdDisplay }}
									<ExternalIcon aria-hidden="true" class="external-icon ml-1 mt-[-1px] inline" />
								</a>
								<span
									v-else-if="
										project.license.id === 'LicenseRef-All-Rights-Reserved' ||
										!project.license.id.includes('LicenseRef')
									"
									class="text-link hover:underline"
									@click="(event) => getLicenseData(event)"
								>
									{{ licenseIdDisplay }}
								</span>
								<span v-else>{{ licenseIdDisplay }}</span>
							</div>
						</div>

						<div
							v-if="project.approved"
							v-tooltip="$dayjs(project.approved).format('MMMM D, YYYY [at] h:mm A')"
							class="details-list__item"
						>
							<CalendarIcon aria-hidden="true" />
							<div>
								{{
									formatMessage(detailsMessages.published, {
										date: publishedDate,
									})
								}}
							</div>
						</div>

						<div
							v-else
							v-tooltip="$dayjs(project.published).format('MMMM D, YYYY [at] h:mm A')"
							class="details-list__item"
						>
							<CalendarIcon aria-hidden="true" />
							<div>
								{{ formatMessage(detailsMessages.created, { date: createdDate }) }}
							</div>
						</div>

						<div
							v-if="project.status === 'processing' && project.queued"
							v-tooltip="$dayjs(project.queued).format('MMMM D, YYYY [at] h:mm A')"
							class="details-list__item"
						>
							<ScaleIcon aria-hidden="true" />
							<div>
								{{
									formatMessage(detailsMessages.submitted, {
										date: submittedDate,
									})
								}}
							</div>
						</div>

						<div
							v-if="versions.length > 0 && project.updated"
							v-tooltip="$dayjs(project.updated).format('MMMM D, YYYY [at] h:mm A')"
							class="details-list__item"
						>
							<VersionIcon aria-hidden="true" />
							<div>
								{{ formatMessage(detailsMessages.updated, { date: updatedDate }) }}
							</div>
						</div>
					</div>
				</div>
			</div>

			<div class="normal-page__content">
				<div class="overflow-x-auto"><NavTabs :links="navLinks" class="mb-4" /></div>
				<NuxtPage
					v-model:project="project"
					v-model:versions="versions"
					v-model:members="members"
					v-model:all-members="allMembers"
					v-model:dependencies="dependencies"
					v-model:organization="organization"
					:current-member="currentMember"
					:reset-project="resetProject"
					:reset-versions="resetVersions"
					:reset-organization="resetOrganization"
					:reset-members="resetMembers"
					:route="route"
					@on-download="triggerDownloadAnimation"
					@delete-version="deleteVersion"
				/>
			</div>
		</div>
	</div>

	<div
		v-if="auth.user && tags.staffRoles.includes(auth.user.role) && showModerationChecklist"
		class="moderation-checklist"
	>
		<ModerationChecklist
			:collapsed="collapsedModerationChecklist"
			@exit="showModerationChecklist = false"
			@toggle-collapsed="collapsedModerationChecklist = !collapsedModerationChecklist"
		/>
	</div>
</template>

<script setup>
import {
	BookmarkIcon,
	BookTextIcon,
	CalendarIcon,
	ChartIcon,
	CheckIcon,
	ChevronRightIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	ExternalIcon,
	GameIcon,
	HeartIcon,
	InfoIcon,
	ListIcon,
	ModrinthIcon,
	MoreVerticalIcon,
	PlusIcon,
	ReportIcon,
	ScaleIcon,
	SearchIcon,
	ServerPlusIcon,
	SettingsIcon,
	VersionIcon,
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	Checkbox,
	commonMessages,
	injectNotificationManager,
	NewModal,
	OverflowMenu,
	PopoutMenu,
	ProjectBackgroundGradient,
	ProjectHeader,
	ProjectSidebarCompatibility,
	ProjectSidebarCreators,
	ProjectSidebarDetails,
	ProjectSidebarLinks,
	provideProjectPageContext,
	ScrollablePanel,
	ServersPromo,
	TagItem,
	useRelativeTime,
} from '@modrinth/ui'
import VersionSummary from '@modrinth/ui/src/components/version/VersionSummary.vue'
import { formatCategory, formatPrice, formatProjectType, renderString } from '@modrinth/utils'
import { IntlFormatted } from '@vintl/vintl/components'
import { useLocalStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import { Tooltip } from 'floating-vue'

import { navigateTo } from '#app'
import Accordion from '~/components/ui/Accordion.vue'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import AutomaticAccordion from '~/components/ui/AutomaticAccordion.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import MessageBanner from '~/components/ui/MessageBanner.vue'
import ModerationChecklist from '~/components/ui/moderation/checklist/ModerationChecklist.vue'
import NavTabs from '~/components/ui/NavTabs.vue'
import ProjectMemberHeader from '~/components/ui/ProjectMemberHeader.vue'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'
import { userCollectProject, userFollowProject } from '~/composables/user.js'
import { useModerationStore } from '~/store/moderation.ts'
import { reportProject } from '~/utils/report-helpers.ts'

const data = useNuxtApp()
const route = useNativeRoute()
const config = useRuntimeConfig()
const moderationStore = useModerationStore()
const notifications = injectNotificationManager()
const { addNotification } = notifications

const auth = await useAuth()
const user = await useUser()

const tags = useGeneratedState()
const flags = useFeatureFlags()
const cosmetics = useCosmetics()

const { locale, formatMessage } = useVIntl()

const settingsModal = ref()
const downloadModal = ref()
const overTheTopDownloadAnimation = ref()

const userSelectedGameVersion = ref(null)
const userSelectedPlatform = ref(null)
const showAllVersions = ref(false)

const gameVersionFilterInput = ref()

const versionFilter = ref('')

const baseId = useId()

const currentGameVersion = computed(() => {
	return (
		userSelectedGameVersion.value ||
		(project.value.game_versions.length === 1 && project.value.game_versions[0])
	)
})

const possibleGameVersions = computed(() => {
	return versions.value
		.filter((x) => !currentPlatform.value || x.loaders.includes(currentPlatform.value))
		.flatMap((x) => x.game_versions)
})

const possiblePlatforms = computed(() => {
	return versions.value
		.filter((x) => !currentGameVersion.value || x.game_versions.includes(currentGameVersion.value))
		.flatMap((x) => x.loaders)
})

const currentPlatform = computed(() => {
	return (
		userSelectedPlatform.value || (project.value.loaders.length === 1 && project.value.loaders[0])
	)
})

const releaseVersions = computed(() => {
	const set = new Set()
	for (const gv of tags.value.gameVersions || []) {
		if (gv?.version && gv.version_type === 'release') set.add(gv.version)
	}
	return set
})

const nonReleaseVersions = computed(() => {
	const set = new Set()
	for (const gv of tags.value.gameVersions || []) {
		if (gv?.version && gv.version_type !== 'release') set.add(gv.version)
	}
	return set
})

function isReleaseGameVersion(ver) {
	if (releaseVersions.value.has(ver)) return true
	if (nonReleaseVersions.value.has(ver)) return false
	return true
}

const showVersionsCheckbox = computed(() => {
	const list = project.value?.game_versions || []
	let hasRelease = false
	let hasNonRelease = false
	for (const v of list) {
		if (isReleaseGameVersion(v)) hasRelease = true
		else hasNonRelease = true
		if (hasRelease && hasNonRelease) return true
	}
	return false
})

function installWithApp() {
	setTimeout(() => {
		getModrinthAppAccordion.value.open()
	}, 1500)
}

const gameVersionAccordion = ref()
const platformAccordion = ref()
const getModrinthAppAccordion = ref()

const formatRelativeTime = useRelativeTime()

const detailsMessages = defineMessages({
	title: {
		id: 'project.about.details.title',
		defaultMessage: 'Details',
	},
	licensed: {
		id: 'project.about.details.licensed',
		defaultMessage: 'Licensed {license}',
	},
	created: {
		id: 'project.about.details.created',
		defaultMessage: 'Created {date}',
	},
	submitted: {
		id: 'project.about.details.submitted',
		defaultMessage: 'Submitted {date}',
	},
	published: {
		id: 'project.about.details.published',
		defaultMessage: 'Published {date}',
	},
	updated: {
		id: 'project.about.details.updated',
		defaultMessage: 'Updated {date}',
	},
})

const messages = defineMessages({
	archivedMessage: {
		id: 'project.status.archived.message',
		defaultMessage:
			'{title} has been archived. {title} will not receive any further updates unless the author decides to unarchive the project.',
	},
	changelogTab: {
		id: 'project.navigation.changelog',
		defaultMessage: 'Changelog',
	},
	createNewCollection: {
		id: 'project.collections.create-new',
		defaultMessage: 'Create new collection',
	},
	createServer: {
		id: 'project.actions.create-server',
		defaultMessage: 'Create a server',
	},
	createServerTooltip: {
		id: 'project.actions.create-server-tooltip',
		defaultMessage: 'Create a server',
	},
	descriptionTab: {
		id: 'project.description.title',
		defaultMessage: 'Description',
	},
	dontHaveModrinthApp: {
		id: 'project.download.no-app',
		defaultMessage: "Don't have Modrinth App?",
	},
	dontShowAgain: {
		id: 'project.actions.dont-show-again',
		defaultMessage: "Don't show again",
	},
	downloadTitle: {
		id: 'project.download.title',
		defaultMessage: 'Download {title}',
	},
	downloadsStat: {
		id: 'project.stats.downloads-label',
		defaultMessage: 'download{count, plural, one {} other {s}}',
	},
	errorLoadingProject: {
		id: 'project.error.loading',
		defaultMessage: 'Error loading project data{message}',
	},
	environmentMigrationMessage: {
		id: 'project.environment.migration.message',
		defaultMessage:
			"We've just overhauled the Environments system on Modrinth and new options are now available. Please visit your project's settings and verify that the metadata is correct.",
	},
	environmentMigrationTitle: {
		id: 'project.environment.migration.title',
		defaultMessage: 'Please review environment metadata',
	},
	environmentMigrationNoPermissionMessage: {
		id: 'project.environment.migration-no-permission.message',
		defaultMessage:
			"We've just overhauled the Environments system on Modrinth and new options are now available. You don't have permission to modify these settings, but please let another member of the project know that the environment metadata needs to be verified.",
	},
	environmentMigrationNoPermissionTitle: {
		id: 'project.environment.migration-no-permission.title',
		defaultMessage: 'Environment metadata needs to be reviewed',
	},
	environmentMigrationLink: {
		id: 'project.environment.migration.learn-more',
		defaultMessage: 'Learn more about this change',
	},
	followersStat: {
		id: 'project.stats.followers-label',
		defaultMessage: 'follower{count, plural, one {} other {s}}',
	},
	galleryTab: {
		id: 'project.gallery.title',
		defaultMessage: 'Gallery',
	},
	gameVersionError: {
		id: 'project.download.game-version-error',
		defaultMessage: 'Error: no game versions found',
	},
	gameVersionLabel: {
		id: 'project.download.game-version',
		defaultMessage: 'Game version: {version}',
	},
	gameVersionTooltip: {
		id: 'project.download.game-version-tooltip',
		defaultMessage: '{title} is only available for {version}',
	},
	gameVersionUnsupportedTooltip: {
		id: 'project.download.game-version-unsupported-tooltip',
		defaultMessage: '{title} does not support {gameVersion} for {platform}',
	},
	installWithModrinthApp: {
		id: 'project.download.install-with-app',
		defaultMessage: 'Install with Modrinth App',
	},
	licenseErrorMessage: {
		id: 'project.license.error',
		defaultMessage: 'License text could not be retrieved.',
	},
	licenseTitle: {
		id: 'project.license.title',
		defaultMessage: 'License',
	},
	licensedLabel: {
		id: 'project.details.licensed',
		defaultMessage: 'Licensed',
	},
	loadingLicenseText: {
		id: 'project.license.loading',
		defaultMessage: 'Loading license text...',
	},
	moderationTab: {
		id: 'project.moderation.title',
		defaultMessage: 'Moderation',
	},
	noCollectionsFound: {
		id: 'project.collections.none-found',
		defaultMessage: 'No collections found.',
	},
	noVersionsAvailable: {
		id: 'project.download.no-versions-available',
		defaultMessage: 'No versions available for {gameVersion} and {platform}.',
	},
	pageNotFound: {
		id: 'project.error.page-not-found',
		defaultMessage: 'The page could not be found',
	},
	platformError: {
		id: 'project.download.platform-error',
		defaultMessage: 'Error: no platforms found',
	},
	platformLabel: {
		id: 'project.download.platform',
		defaultMessage: 'Platform: {platform}',
	},
	platformTooltip: {
		id: 'project.download.platform-tooltip',
		defaultMessage: '{title} is only available for {platform}',
	},
	platformUnsupportedTooltip: {
		id: 'project.download.platform-unsupported-tooltip',
		defaultMessage: '{title} does not support {platform} for {gameVersion}',
	},
	projectIconUpdated: {
		id: 'project.notification.icon-updated.title',
		defaultMessage: 'Project icon updated',
	},
	projectIconUpdatedMessage: {
		id: 'project.notification.icon-updated.message',
		defaultMessage: "Your project's icon has been updated.",
	},
	projectNotFound: {
		id: 'project.error.project-not-found',
		defaultMessage: 'Project not found',
	},
	projectUpdated: {
		id: 'project.notification.updated.title',
		defaultMessage: 'Project updated',
	},
	projectUpdatedMessage: {
		id: 'project.notification.updated.message',
		defaultMessage: 'Your project has been updated.',
	},
	reviewEnvironmentSettings: {
		id: 'project.environment.migration.review-button',
		defaultMessage: 'Review environment settings',
	},
	reviewProject: {
		id: 'project.actions.review-project',
		defaultMessage: 'Review project',
	},
	searchGameVersions: {
		id: 'project.download.search-game-versions',
		defaultMessage: 'Search game versions...',
	},
	searchGameVersionsLabel: {
		id: 'project.download.search-game-versions-label',
		defaultMessage: 'Search game versions...',
	},
	selectGameVersion: {
		id: 'project.download.select-game-version',
		defaultMessage: 'Select game version',
	},
	selectPlatform: {
		id: 'project.download.select-platform',
		defaultMessage: 'Select platform',
	},
	serversPromoDescription: {
		id: 'project.actions.servers-promo.description',
		defaultMessage: 'Modrinth Hosting is the easiest way to play with your friends without hassle!',
	},
	serversPromoPricing: {
		id: 'project.actions.servers-promo.pricing',
		defaultMessage: 'Starting at {price}<small> / month</small>',
	},
	serversPromoTitle: {
		id: 'project.actions.servers-promo.title',
		defaultMessage: 'Create a server',
	},
	settingsTitle: {
		id: 'project.settings.title',
		defaultMessage: 'Settings',
	},
	showAllVersions: {
		id: 'project.download.show-all-versions',
		defaultMessage: 'Show all versions',
	},
	versionsTab: {
		id: 'project.versions.title',
		defaultMessage: 'Versions',
	},
	visitProjectsDashboard: {
		id: 'project.settings.visit-dashboard',
		defaultMessage: 'Visit projects dashboard',
	},
})

const modalLicense = ref(null)
const licenseText = ref('')

const createdDate = computed(() =>
	project.value.published ? formatRelativeTime(project.value.published) : 'unknown',
)
const submittedDate = computed(() =>
	project.value.queued ? formatRelativeTime(project.value.queued) : 'unknown',
)
const publishedDate = computed(() =>
	project.value.approved ? formatRelativeTime(project.value.approved) : 'unknown',
)
const updatedDate = computed(() =>
	project.value.updated ? formatRelativeTime(project.value.updated) : 'unknown',
)

const licenseIdDisplay = computed(() => {
	const id = project.value.license.id

	if (id === 'LicenseRef-All-Rights-Reserved') {
		return 'ARR'
	} else if (id.includes('LicenseRef')) {
		return id.replaceAll('LicenseRef-', '').replaceAll('-', ' ')
	} else {
		return id
	}
})

async function getLicenseData(event) {
	modalLicense.value.show(event)

	try {
		const text = await useBaseFetch(`tag/license/${project.value.license.id}`)
		licenseText.value = text.body || formatMessage(messages.licenseErrorMessage)
	} catch {
		licenseText.value = formatMessage(messages.licenseErrorMessage)
	}
}

const filteredVersions = computed(() => {
	return versions.value.filter(
		(x) =>
			x.game_versions.includes(currentGameVersion.value) &&
			(x.loaders.includes(currentPlatform.value) || project.value.project_type === 'resourcepack'),
	)
})

const filteredRelease = computed(() => {
	return filteredVersions.value.find((x) => x.version_type === 'release')
})

const filteredBeta = computed(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'beta' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))),
	)
})

const filteredAlpha = computed(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'alpha' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))) &&
			(!filteredBeta.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredBeta.value.date_published))),
	)
})

const displayCollectionsSearch = ref('')
const collections = computed(() =>
	user.value && user.value.collections
		? user.value.collections.filter((x) =>
				x.name.toLowerCase().includes(displayCollectionsSearch.value.toLowerCase()),
			)
		: [],
)

if (
	!route.params.id ||
	!(
		tags.value.projectTypes.find((x) => x.id === route.params.type) ||
		route.params.type === 'project'
	)
) {
	throw createError({
		fatal: true,
		statusCode: 404,
		message: formatMessage(messages.pageNotFound),
	})
}

const projectId = ref(route.params.id)

let project,
	projectV3,
	resetProjectV2,
	resetProjectV3,
	allMembers,
	resetMembers,
	dependencies,
	versions,
	resetVersions,
	organization,
	resetOrganization,
	projectV2Error,
	projectV3Error,
	membersError,
	dependenciesError,
	versionsError
try {
	;[
		{ data: project, error: projectV2Error, refresh: resetProjectV2 },
		{ data: projectV3, error: projectV3Error, refresh: resetProjectV3 },
		{ data: allMembers, error: membersError, refresh: resetMembers },
		{ data: dependencies, error: dependenciesError },
		{ data: versions, error: versionsError, refresh: resetVersions },
		{ data: organization, refresh: resetOrganization },
	] = await Promise.all([
		useAsyncData(`project/${projectId.value}`, () => useBaseFetch(`project/${projectId.value}`), {
			transform: (project) => {
				if (project) {
					project.actualProjectType = JSON.parse(JSON.stringify(project.project_type))
					project.project_type = data.$getProjectTypeForUrl(
						project.project_type,
						project.loaders,
						tags.value,
					)
					projectId.value = project.id
				}
				return project
			},
		}),
		useAsyncData(`projectV3/${projectId.value}`, () =>
			useBaseFetch(`project/${projectId.value}`, {
				apiVersion: 3,
			}),
		),
		useAsyncData(
			`project/${projectId.value}/members`,
			() => useBaseFetch(`project/${projectId.value}/members`, { apiVersion: 3 }),
			{
				transform: (members) => {
					members.forEach((it, index) => {
						members[index].avatar_url = it.user.avatar_url
						members[index].name = it.user.username
					})

					return members
				},
			},
		),
		useAsyncData(`project/${projectId.value}/dependencies`, () =>
			useBaseFetch(`project/${projectId.value}/dependencies`, {}),
		),
		useAsyncData(`project/${projectId.value}/version`, () =>
			useBaseFetch(`project/${projectId.value}/version`),
		),
		useAsyncData(`project/${projectId.value}/organization`, () =>
			useBaseFetch(`project/${projectId.value}/organization`, { apiVersion: 3 }),
		),
	])

	await updateProjectRoute()

	versions = shallowRef(toRaw(versions))
} catch (err) {
	throw createError({
		fatal: true,
		statusCode: err.statusCode ?? 500,
		message: formatMessage(messages.errorLoadingProject, {
			message: err.message ? `: ${err.message}` : '',
		}),
	})
}

async function updateProjectRoute() {
	if (project.value && route.params.id !== project.value.slug) {
		await navigateTo(
			{
				name: route.name,
				params: {
					...route.params,
					id: project.value.slug,
				},
				query: route.query,
				hash: route.hash,
			},
			{ replace: true },
		)
	}
}

async function resetProject() {
	await resetProjectV2()
	await resetProjectV3()
}

function handleError(err, project = false) {
	if (err.value && err.value.statusCode) {
		throw createError({
			fatal: true,
			statusCode: err.value.statusCode,
			message:
				err.value.statusCode === 404 && project
					? formatMessage(messages.projectNotFound)
					: err.value.message,
		})
	}
}

handleError(projectV2Error, true)
handleError(projectV3Error)
handleError(membersError)
handleError(dependenciesError)
handleError(versionsError)

if (!project.value) {
	throw createError({
		fatal: true,
		statusCode: 404,
		message: formatMessage(messages.projectNotFound),
	})
}

if (project.value.project_type !== route.params.type || route.params.id !== project.value.slug) {
	let path = route.fullPath.split('/')
	path.splice(0, 3)
	path = path.filter((x) => x)

	await navigateTo(
		`/${project.value.project_type}/${project.value.slug}${
			path.length > 0 ? `/${path.join('/')}` : ''
		}`,
		{ redirectCode: 301, replace: true },
	)
}

// Members should be an array of all members, without the accepted ones, and with the user with the Owner role at the start
// The rest of the members should be sorted by role, then by name
const members = computed(() => {
	const acceptedMembers = allMembers.value.filter((x) => x.accepted)
	const owner = acceptedMembers.find((x) =>
		organization.value
			? organization.value.members.some(
					(orgMember) => orgMember.user.id === x.user.id && orgMember.is_owner,
				)
			: x.is_owner,
	)

	const rest = acceptedMembers.filter((x) => !owner || x.user.id !== owner.user.id) || []

	rest.sort((a, b) => {
		if (a.role === b.role) {
			return a.user.username.localeCompare(b.user.username)
		} else {
			return a.role.localeCompare(b.role)
		}
	})

	return owner ? [owner, ...rest] : rest
})

const isMember = computed(
	() => auth.value.user && allMembers.value.some((x) => x.user.id === auth.value.user.id),
)

const currentMember = computed(() => {
	let val = auth.value.user ? allMembers.value.find((x) => x.user.id === auth.value.user.id) : null

	if (!val && auth.value.user && organization.value && organization.value.members) {
		val = organization.value.members.find((x) => x.user.id === auth.value.user.id)
	}

	if (!val && auth.value.user && tags.value.staffRoles.includes(auth.value.user.role)) {
		val = {
			team_id: project.team_id,
			user: auth.value.user,
			role: auth.value.role,
			permissions: auth.value.user.role === 'admin' ? 1023 : 12,
			accepted: true,
			payouts_split: 0,
			avatar_url: auth.value.user.avatar_url,
			name: auth.value.user.username,
		}
	}

	return val
})

const hasEditDetailsPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (currentMember.value?.permissions & EDIT_DETAILS) === EDIT_DETAILS
})

versions.value = data.$computeVersions(versions.value, allMembers.value)

const projectTypeDisplay = computed(() =>
	formatProjectType(
		data.$getProjectTypeForDisplay(project.value.project_type, project.value.loaders),
	),
)

const following = computed(() => {
	if (!user.value?.follows) {
		return false
	}
	return !!user.value.follows.find((x) => x.id === project.value.id)
})

const title = computed(() => `${project.value.title} - Minecraft ${projectTypeDisplay.value}`)
const description = computed(
	() =>
		`${project.value.description} - Download the Minecraft ${projectTypeDisplay.value} ${
			project.value.title
		} by ${members.value.find((x) => x.is_owner)?.user?.username || 'a creator'} on Modrinth`,
)

const canCreateServerFrom = computed(() => {
	return project.value.project_type === 'modpack' && project.value.server_side !== 'unsupported'
})

if (!route.name.startsWith('type-id-settings')) {
	useSeoMeta({
		title: () => title.value,
		description: () => description.value,
		ogTitle: () => title.value,
		ogDescription: () => project.value.description,
		ogImage: () => project.value.icon_url ?? 'https://cdn.modrinth.com/placeholder.png',
		robots: () =>
			project.value.status === 'approved' || project.value.status === 'archived'
				? 'all'
				: 'noindex',
	})
}

const onUserCollectProject = useClientTry(userCollectProject)

const { version, loader } = route.query

if (
	project.value.game_versions.length > 0 &&
	project.value.game_versions.every((v) => !isReleaseGameVersion(v))
) {
	showAllVersions.value = true
}

if (version !== undefined && project.value.game_versions.includes(version)) {
	userSelectedGameVersion.value = version
}

if (loader !== undefined && project.value.loaders.includes(loader)) {
	userSelectedPlatform.value = loader
}

watch(downloadModal, (modal) => {
	if (!modal) return

	// route.hash returns everything in the hash string, including the # itself
	if (route.hash === '#download') {
		modal.show()
	}
})

async function setProcessing() {
	startLoading()

	try {
		await useBaseFetch(`project/${project.value.id}`, {
			method: 'PATCH',
			body: {
				status: 'processing',
			},
		})

		project.value.status = 'processing'
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	stopLoading()
}

async function patchProject(resData, quiet = false) {
	let result = false
	startLoading()

	try {
		await useBaseFetch(`project/${project.value.id}`, {
			method: 'PATCH',
			body: resData,
		})

		for (const key in resData) {
			project.value[key] = resData[key]
		}

		await updateProjectRoute()

		if ('license_id' in resData) {
			project.value.license.id = resData.license_id
		}
		if ('license_url' in resData) {
			project.value.license.url = resData.license_url
		}

		result = true
		if (!quiet) {
			addNotification({
				title: formatMessage(messages.projectUpdated),
				text: formatMessage(messages.projectUpdatedMessage),
				type: 'success',
			})
			window.scrollTo({ top: 0, behavior: 'smooth' })
		}
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
		window.scrollTo({ top: 0, behavior: 'smooth' })
	}

	stopLoading()

	return result
}

async function patchIcon(icon) {
	let result = false
	startLoading()

	try {
		await useBaseFetch(
			`project/${project.value.id}/icon?ext=${
				icon.type.split('/')[icon.type.split('/').length - 1]
			}`,
			{
				method: 'PATCH',
				body: icon,
			},
		)
		await resetProject()
		result = true
		addNotification({
			title: formatMessage(messages.projectIconUpdated),
			text: formatMessage(messages.projectIconUpdatedMessage),
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})

		window.scrollTo({ top: 0, behavior: 'smooth' })
	}

	stopLoading()
	return result
}

async function updateMembers() {
	allMembers.value = await useAsyncData(
		`project/${projectId.value}/members`,
		() => useBaseFetch(`project/${projectId.value}/members`),
		{
			transform: (members) => {
				members.forEach((it, index) => {
					members[index].avatar_url = it.user.avatar_url
					members[index].name = it.user.username
				})

				return members
			},
		},
	)
}

async function copyId() {
	await navigator.clipboard.writeText(project.value.id)
}

async function copyPermalink() {
	await navigator.clipboard.writeText(`${config.public.siteUrl}/project/${project.value.id}`)
}

const collapsedChecklist = ref(false)

const showModerationChecklist = useLocalStorage(
	`show-moderation-checklist-${project.value.id}`,
	false,
)
const collapsedModerationChecklist = useLocalStorage('collapsed-moderation-checklist', false)

if (import.meta.client && history && history.state && history.state.showChecklist) {
	showModerationChecklist.value = true
}

function closeDownloadModal(event) {
	downloadModal.value.hide(event)
	userSelectedPlatform.value = null
	userSelectedGameVersion.value = null
	showAllVersions.value = false
}

function triggerDownloadAnimation() {
	overTheTopDownloadAnimation.value = true
	setTimeout(() => (overTheTopDownloadAnimation.value = false), 500)
}

function onDownload(event) {
	triggerDownloadAnimation()
	setTimeout(() => {
		closeDownloadModal(event)
	}, 400)
}

async function deleteVersion(id) {
	if (!id) return

	startLoading()

	await useBaseFetch(`version/${id}`, {
		method: 'DELETE',
	})

	versions.value = versions.value.filter((x) => x.id !== id)

	stopLoading()
}

const navLinks = computed(() => {
	const projectUrl = `/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`

	return [
		{
			label: formatMessage(messages.descriptionTab),
			href: projectUrl,
		},
		{
			label: formatMessage(messages.galleryTab),
			href: `${projectUrl}/gallery`,
			shown: project.value.gallery.length > 0 || !!currentMember.value,
		},
		{
			label: formatMessage(messages.changelogTab),
			href: `${projectUrl}/changelog`,
			shown: versions.value.length > 0,
		},
		{
			label: formatMessage(messages.versionsTab),
			href: `${projectUrl}/versions`,
			shown: versions.value.length > 0 || !!currentMember.value,
			subpages: [`${projectUrl}/version/`],
		},
		{
			label: formatMessage(messages.moderationTab),
			href: `${projectUrl}/moderation`,
			shown: !!currentMember.value,
		},
	]
})

provideProjectPageContext({
	projectV2: project,
	projectV3,
	refreshProject: resetProject,
	refreshVersions: resetVersions,
	currentMember,
})
</script>

<style lang="scss" scoped>
.settings-header {
	display: flex;
	flex-direction: row;
	gap: var(--spacing-card-sm);
	align-items: center;
	margin-bottom: var(--spacing-card-bg);

	.settings-header__icon {
		flex-shrink: 0;
	}

	.settings-header__text {
		h1 {
			font-size: var(--font-size-md);
			margin-top: 0;
			margin-bottom: var(--spacing-card-sm);
		}
	}
}

.popout-checkbox {
	padding: var(--gap-sm) var(--gap-md);
	white-space: nowrap;

	&:hover {
		filter: brightness(0.95);
	}
}

.popout-heading {
	padding: var(--gap-sm) var(--gap-md);
	padding-bottom: 0;
	font-size: var(--font-size-nm);
	color: var(--color-secondary);
}

.collection-button {
	margin: var(--gap-sm) var(--gap-md);
	white-space: nowrap;
}

.menu-text {
	padding: 0 var(--gap-md);
	font-size: var(--font-size-nm);
	color: var(--color-secondary);
}

.menu-search {
	margin: var(--gap-sm) var(--gap-md);
	width: calc(100% - var(--gap-md) * 2);
}

.collections-list {
	max-height: 40rem;
	overflow-y: auto;
	background-color: var(--color-bg);
	border-radius: var(--radius-md);
	margin: var(--gap-sm) var(--gap-md);
	padding: var(--gap-sm);
}

.normal-page__info:empty {
	display: none;
}

:deep(.accordion-with-bg) {
	@apply rounded-2xl bg-bg p-2;
	--scrollable-pane-bg: var(--color-bg);
}

.over-the-top-download-animation {
	position: fixed;
	z-index: 100;
	inset: 0;
	display: flex;
	justify-content: center;
	align-items: center;
	pointer-events: none;
	scale: 0.5;
	transition: all 0.5s ease-out;
	opacity: 1;

	&.animation-hidden {
		scale: 0.8;
		opacity: 0;

		.animation-ring-1 {
			width: 25rem;
			height: 25rem;
		}

		.animation-ring-2 {
			width: 50rem;
			height: 50rem;
		}

		.animation-ring-3 {
			width: 100rem;
			height: 100rem;
		}
	}

	> div {
		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;
		width: fit-content;
		height: fit-content;

		> * {
			position: absolute;
			scale: 1;
			transition: all 0.2s ease-out;
			width: 20rem;
			height: 20rem;
		}
	}
}

@media (hover: none) and (max-width: 767px) {
	.modrinth-app-section {
		display: none;
	}
}

.servers-popup {
	box-shadow:
		0 0 12px 1px rgba(0, 175, 92, 0.6),
		var(--shadow-floating);

	&::before {
		width: 0;
		height: 0;
		border-left: 6px solid transparent;
		border-right: 6px solid transparent;
		border-bottom: 6px solid var(--color-button-bg);
		content: ' ';
		position: absolute;
		top: -7px;
		left: 17px;
	}
	&::after {
		width: 0;
		height: 0;
		border-left: 5px solid transparent;
		border-right: 5px solid transparent;
		border-bottom: 5px solid var(--color-raised-bg);
		content: ' ';
		position: absolute;
		top: -5px;
		left: 18px;
	}
}

.moderation-checklist {
	position: fixed;
	bottom: 1rem;
	right: 1rem;
	overflow-y: auto;
	z-index: 50;

	> div {
		box-shadow: 0 0 15px rgba(0, 0, 0, 0.3);
	}
}
</style>
