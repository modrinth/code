<template>
	<div
		ref="main_page"
		class="layout"
		:class="{
			'expanded-mobile-nav': isBrowseMenuOpen,
			'modrinth-parent__no-modal-blurs': !cosmetics.advancedRendering,
		}"
	>
		<div class="pointer-events-none fixed inset-0 z-[-1]">
			<div id="fixed-background-teleport" class="relative"></div>
		</div>
		<div class="pointer-events-none absolute inset-0 z-[-1]">
			<div id="absolute-background-teleport" class="relative"></div>
		</div>
		<div
			class="pride-backdrop pointer-events-none absolute inset-0 z-[-1]"
			:class="{ shown: showPrideBackdrop }"
		></div>
		<div class="pointer-events-none absolute inset-0 z-50">
			<div
				class="over-the-top-random-animation"
				:style="{ '--_r-count': rCount }"
				:class="{ threshold: rCount > 20, 'rings-expand': rCount >= 40 }"
			>
				<div>
					<div
						class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
					></div>
					<div
						class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
					></div>
					<div
						class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight text-9xl font-extrabold text-contrast"
					>
						?
					</div>
				</div>
			</div>
		</div>
		<RussiaBanner v-if="flags.showAllBanners || isRussia" />
		<TaxIdMismatchBanner v-if="flags.showAllBanners || showTinMismatchBanner" />
		<TaxComplianceBanner v-if="flags.showAllBanners || showTaxComplianceBanner" />
		<VerifyEmailBanner
			v-if="
				flags.showAllBanners ||
				(auth.user && !auth.user.email_verified && route.path !== '/auth/verify-email')
			"
			:has-email="!!auth?.user?.email"
		/>
		<SubscriptionPaymentFailedBanner
			v-if="
				flags.showAllBanners ||
				(user.subscriptions.some((x) => x.status === 'payment-failed') &&
					route.path !== '/settings/billing')
			"
		/>
		<PreviewBanner
			v-if="
				flags.showAllBanners || (config.public.buildEnv === 'production' && config.public.preview)
			"
		/>
		<StagingBanner
			v-if="
				flags.showAllBanners ||
				config.public.apiBaseUrl.startsWith('https://staging-api.modrinth.com')
			"
		/>
		<GeneratedStateErrorsBanner
			:errors="generatedStateErrors"
			:api-url="config.public.apiBaseUrl"
		/>
		<ViewOnModrinthBanner />
		<header
			class="desktop-only relative z-[5] mx-auto grid max-w-[1280px] grid-cols-[1fr_auto] items-center gap-2 px-6 py-4 lg:grid-cols-[auto_1fr_auto]"
		>
			<div>
				<NuxtLink
					to="/"
					:aria-label="formatMessage(messages.modrinthHomePage)"
					class="group hover:brightness-[--hover-brightness] focus-visible:brightness-[--hover-brightness]"
				>
					<TextLogo
						aria-hidden="true"
						class="h-7 w-auto text-contrast transition-transform group-active:scale-[0.98]"
					/>
				</NuxtLink>
			</div>
			<div
				class="col-span-2 row-start-2 flex justify-center lg:col-span-1 lg:row-start-auto"
				:class="{ 'gap-4': !flags.projectTypesPrimaryNav }"
			>
				<template v-if="flags.projectTypesPrimaryNav">
					<ButtonLink
						type="quiet"
						to="/discover/mods"
						:class="
							route.name === 'discover-mods' || route.path.startsWith('/mod/')
								? (route.name === 'discover-mods' ? 'main-nav-primary' : 'main-nav-secondary') ===
									'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<BoxIcon aria-hidden="true" />
						{{ formatMessage(commonProjectTypeCategoryMessages.mod) }}
					</ButtonLink>
					<ButtonLink
						type="quiet"
						to="/discover/resourcepacks"
						:class="
							route.name === 'discover-resourcepacks' || route.path.startsWith('/resourcepack/')
								? (route.name === 'discover-resourcepacks'
										? 'main-nav-primary'
										: 'main-nav-secondary') === 'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<PaintbrushIcon aria-hidden="true" />
						{{ formatMessage(commonProjectTypeCategoryMessages.resourcepack) }}
					</ButtonLink>
					<ButtonLink
						type="quiet"
						to="/discover/datapacks"
						:class="
							route.name === 'discover-datapacks' || route.path.startsWith('/datapack/')
								? (route.name === 'discover-datapacks'
										? 'main-nav-primary'
										: 'main-nav-secondary') === 'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<BracesIcon aria-hidden="true" />
						{{ formatMessage(commonProjectTypeCategoryMessages.datapack) }}
					</ButtonLink>
					<ButtonLink
						type="quiet"
						to="/discover/shaders"
						:class="
							route.name === 'discover-shaders' || route.path.startsWith('/shader/')
								? (route.name === 'discover-shaders'
										? 'main-nav-primary'
										: 'main-nav-secondary') === 'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<GlassesIcon aria-hidden="true" />
						{{ formatMessage(commonProjectTypeCategoryMessages.shader) }}
					</ButtonLink>
					<ButtonLink
						type="quiet"
						to="/discover/modpacks"
						:class="
							route.name === 'discover-modpacks' || route.path.startsWith('/modpack/')
								? (route.name === 'discover-modpacks'
										? 'main-nav-primary'
										: 'main-nav-secondary') === 'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<PackageOpenIcon aria-hidden="true" />
						{{ formatMessage(commonProjectTypeCategoryMessages.modpack) }}
					</ButtonLink>
					<ButtonLink
						type="quiet"
						to="/discover/plugins"
						:class="
							route.name === 'discover-plugins' || route.path.startsWith('/plugin/')
								? (route.name === 'discover-plugins'
										? 'main-nav-primary'
										: 'main-nav-secondary') === 'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<PlugIcon aria-hidden="true" />
						{{ formatMessage(commonProjectTypeCategoryMessages.plugin) }}
					</ButtonLink>
					<ButtonLink
						type="quiet"
						to="/discover/servers"
						:class="
							route.name === 'discover-servers' || route.path.startsWith('/server/')
								? (route.name === 'discover-servers'
										? 'main-nav-primary'
										: 'main-nav-secondary') === 'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<ServerIcon aria-hidden="true" />
						{{ formatMessage(commonProjectTypeCategoryMessages.server) }}
					</ButtonLink>
				</template>
				<template v-else>
					<TeleportOverflowMenu
						type="quiet"
						:label="formatMessage(commonMessages.moreOptionsButton)"
						hoverable
						:options="[
							{
								id: 'mods',
								label: formatMessage(commonProjectTypeCategoryMessages.mod),
								type: 'link',
								to: '/discover/mods',
							},
							{
								id: 'resourcepacks',
								label: formatMessage(commonProjectTypeCategoryMessages.resourcepack),
								type: 'link',
								to: '/discover/resourcepacks',
							},
							{
								id: 'datapacks',
								label: formatMessage(commonProjectTypeCategoryMessages.datapack),
								type: 'link',
								to: '/discover/datapacks',
							},
							{
								id: 'shaders',
								label: formatMessage(commonProjectTypeCategoryMessages.shader),
								type: 'link',
								to: '/discover/shaders',
							},
							{
								id: 'modpacks',
								label: formatMessage(commonProjectTypeCategoryMessages.modpack),
								type: 'link',
								to: '/discover/modpacks',
							},
							{
								id: 'plugins',
								label: formatMessage(commonProjectTypeCategoryMessages.plugin),
								type: 'link',
								to: '/discover/plugins',
							},
							{
								id: 'servers',
								label: formatMessage(commonProjectTypeCategoryMessages.server),
								type: 'link',
								to: '/discover/servers',
							},
						]"
						:class="[
							'!w-auto !rounded-xl !px-2.5',
							isDiscovering
								? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
								: isDiscoveringSubpage
									? '!bg-[var(--color-button-bg)] !text-contrast'
									: '',
						]"
					>
						<BoxIcon
							v-if="route.name === 'discover-mods' || route.path.startsWith('/mod/')"
							aria-hidden="true"
						/>
						<PaintbrushIcon
							v-else-if="
								route.name === 'discover-resourcepacks' || route.path.startsWith('/resourcepack/')
							"
							aria-hidden="true"
						/>
						<BracesIcon
							v-else-if="route.name === 'discover-datapacks' || route.path.startsWith('/datapack/')"
							aria-hidden="true"
						/>
						<PackageOpenIcon
							v-else-if="route.name === 'discover-modpacks' || route.path.startsWith('/modpack/')"
							aria-hidden="true"
						/>
						<GlassesIcon
							v-else-if="route.name === 'discover-shaders' || route.path.startsWith('/shader/')"
							aria-hidden="true"
						/>
						<PlugIcon
							v-else-if="route.name === 'discover-plugins' || route.path.startsWith('/plugin/')"
							aria-hidden="true"
						/>
						<ServerIcon
							v-else-if="route.name === 'discover-servers' || route.path.startsWith('/server/')"
							aria-hidden="true"
						/>
						<CompassIcon v-else aria-hidden="true" />
						<span class="hidden md:contents">{{
							formatMessage(commonMessages.discoverContentLabel)
						}}</span>
						<span class="contents md:hidden">{{ formatMessage(navMenuMessages.discover) }}</span>
						<DropdownIcon aria-hidden="true" class="h-5 w-5" />

						<template #mods>
							<BoxIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.mod) }}
						</template>
						<template #resourcepacks>
							<PaintbrushIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.resourcepack) }}
						</template>
						<template #datapacks>
							<BracesIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.datapack) }}
						</template>
						<template #plugins>
							<PlugIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.plugin) }}
						</template>
						<template #shaders>
							<GlassesIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.shader) }}
						</template>
						<template #modpacks>
							<PackageOpenIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.modpack) }}
						</template>
						<template #servers>
							<ServerIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.server) }}
						</template>
					</TeleportOverflowMenu>
					<ButtonLink
						type="quiet"
						to="/hosting"
						:class="
							route.name?.startsWith('hosting') ||
							(route.name?.startsWith('discover-') && !!route.query.sid)
								? (route.name === 'hosting' ? 'main-nav-primary' : 'main-nav-secondary') ===
									'main-nav-primary'
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<ServerStackIcon aria-hidden="true" />
						{{ formatMessage(navMenuMessages.hostAServer) }}
					</ButtonLink>
					<ButtonLink
						type="quiet"
						to="/app"
						:class="
							route.name === 'app'
								? true
									? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)] [&>svg]:!text-[var(--color-button-text-selected)]'
									: '!bg-[var(--color-button-bg)] !text-contrast'
								: ''
						"
					>
						<DownloadIcon aria-hidden="true" />
						<span class="hidden md:contents">{{
							formatMessage(navMenuMessages.getModrinthApp)
						}}</span>
						<span class="contents md:hidden">{{ formatMessage(navMenuMessages.modrinthApp) }}</span>
					</ButtonLink>
				</template>
			</div>
			<div class="flex items-center gap-1">
				<TeleportOverflowMenu
					v-if="auth.user && isStaff(auth.user)"
					type="quiet"
					:icon-only="false"
					:label="formatMessage(messages.createNew)"
					class="btn-dropdown-animation !gap-1 !rounded-xl !px-2"
					:options="[
						{
							id: 'review-projects',
							label: formatMessage(messages.reviewProjects),
							icon: ScaleIcon,
							type: 'link',
							to: '/moderation',
							tone: 'orange',
						},
						{
							id: 'tech-review',
							label: formatMessage(messages.techReview),
							icon: ShieldAlertIcon,
							type: 'link',
							to: '/moderation/technical-review',
							tone: 'orange',
						},
						{
							id: 'review-reports',
							label: formatMessage(messages.reports),
							icon: ReportIcon,
							type: 'link',
							to: '/moderation/reports',
							tone: 'orange',
						},
						{
							id: 'external-projects',
							label: formatMessage(messages.externalProjects),
							icon: GlobeIcon,
							type: 'link',
							to: '/moderation/external-projects',
							tone: 'orange',
						},
						{
							id: 'global-traces',
							label: 'Global traces',
							icon: HashIcon,
							type: 'link',
							to: '/moderation/global-traces',
							tone: 'orange',
						},
						{ type: 'divider' },
						{
							id: 'file-lookup',
							label: 'File lookup',
							icon: FileSearchCornerIcon,
							type: 'link',
							to: '/admin/file_lookup',
						},
						{
							id: 'user-lookup',
							label: 'User lookup',
							icon: UserSearchIcon,
							type: 'link',
							to: '/admin/user_email',
							shown: isAdmin(auth.user),
						},
						{
							type: 'divider',
							shown: isAdmin(auth.user),
						},
						{
							id: 'servers-lookup',
							label: 'Server lookup',
							icon: ServerSearchIcon,
							type: 'link',
							to: '/admin/servers/lookup',
							shown: isAdmin(auth.user),
						},
						{
							id: 'servers-notices',
							label: 'Server notices',
							icon: IssuesIcon,
							type: 'link',
							to: '/admin/servers/notices',
							shown: isAdmin(auth.user),
						},
						{
							id: 'servers-transfers',
							label: 'Server transfers',
							icon: TransferIcon,
							type: 'link',
							to: '/admin/servers/transfers',
							shown: isAdmin(auth.user),
						},
						{
							id: 'servers-nodes',
							label: 'Credit server nodes',
							icon: ServerIcon,
							action: (event) => $refs.modal_batch_credit.show(event),
							shown: isAdmin(auth.user),
						},
						{
							type: 'divider',
							shown: isAdmin(auth.user),
						},
						{
							id: 'affiliates',
							label: 'Affiliate links',
							icon: AffiliateIcon,
							type: 'link',
							to: '/admin/affiliates',
							shown: isAdmin(auth.user),
						},
						{
							id: 'analytics-events',
							label: 'Analytics events',
							icon: ChartIcon,
							type: 'link',
							to: '/admin/analytics/events',
							shown: isAdmin(auth.user),
						},
						{ type: 'divider' },
						{
							id: 'email-templates',
							label: 'Email templates',
							icon: MailIcon,
							type: 'link',
							to: '/admin/emails',
						},
						{
							id: 'document-templates',
							label: 'Document templates',
							icon: BookOpenIcon,
							type: 'link',
							to: '/admin/docs',
						},
					]"
				>
					<ModrinthIcon aria-hidden="true" />
					<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
				</TeleportOverflowMenu>
				<TeleportOverflowMenu
					v-if="auth.user"
					type="quiet"
					:icon-only="false"
					:label="formatMessage(messages.createNew)"
					class="btn-dropdown-animation !gap-1 !rounded-xl !px-2"
					:options="[
						{
							id: 'new-project',
							label: formatMessage(messages.newProject),
							icon: BoxPlusIcon,
							action: (event) => requireVerifiedEmail(() => $refs.modal_creation.show(event)),
						},
						{
							id: 'new-server-project',
							label: formatMessage(messages.newServerProject),
							icon: ServerPlusIcon,
							action: (event) =>
								requireVerifiedEmail(() => $refs.modal_creation.show(event, { type: 'server' })),
						},
						{
							id: 'new-collection',
							label: formatMessage(messages.newCollection),
							icon: CollectionPlusIcon,
							action: (event) =>
								requireVerifiedEmail(() => $refs.modal_collection_creation.show(event)),
						},
						{ type: 'divider' },
						{
							id: 'new-organization',
							label: formatMessage(messages.newOrganization),
							icon: OrganizationPlusIcon,
							action: (event) =>
								requireVerifiedEmail(() => $refs.modal_organization_creation.show(event)),
						},
					]"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.publish) }}
				</TeleportOverflowMenu>
				<TeleportOverflowMenu
					v-if="auth.user"
					type="quiet"
					size="lg"
					interaction="none"
					:icon-only="false"
					:label="formatMessage(commonMessages.moreOptionsButton)"
					class="btn-dropdown-animation !gap-1 !rounded-xl !px-2 !pr-1"
					:options="userMenuOptions"
				>
					<Avatar :src="auth.user.avatar_url" aria-hidden="true" circle />
					<DropdownIcon class="h-5 w-5 text-secondary" />
					<template
						v-for="account in accountSwitcherAccounts"
						:key="account.id"
						#[account.optionId]
					>
						<Avatar :src="account.avatarUrl" size="1.25rem" aria-hidden="true" circle />
						{{ account.username }}
						<UserRoleIcon :role="account.role" />
					</template>
				</TeleportOverflowMenu>
				<template v-else>
					<TeleportOverflowMenu
						v-if="accountSwitcherAccounts.length > 0"
						type="colored"
						color="brand"
						:icon-only="false"
						:label="formatMessage(commonMessages.signInButton)"
						class="btn-dropdown-animation !gap-1 !pr-1"
						:options="accountSwitcherOptions"
					>
						<LogInIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.signInButton) }}
						<DropdownIcon class="h-5 w-5" />
						<template
							v-for="account in accountSwitcherAccounts"
							:key="account.id"
							#[account.optionId]
						>
							<Avatar :src="account.avatarUrl" size="1.25rem" aria-hidden="true" circle />
							{{ account.username }}
							<UserRoleIcon :role="account.role" />
						</template>
					</TeleportOverflowMenu>
					<ButtonLink v-else type="colored" color="brand" :to="signInRouteObj">
						<LogInIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.signInButton) }}
					</ButtonLink>
					<ButtonLink
						v-tooltip="formatMessage(commonMessages.settingsLabel)"
						to="/settings"
						class="!w-9 !rounded-full !px-0"
					>
						<SettingsIcon :aria-label="formatMessage(commonMessages.settingsLabel)" />
					</ButtonLink>
				</template>
			</div>
		</header>
		<header class="mobile-navigation mobile-only">
			<div
				class="nav-menu nav-menu-browse"
				:class="{ expanded: isBrowseMenuOpen }"
				@focusin="isBrowseMenuOpen = true"
				@focusout="isBrowseMenuOpen = false"
			>
				<div class="links cascade-links">
					<ButtonLink
						v-for="navRoute in navRoutes"
						:key="navRoute.href"
						:to="navRoute.href"
						class="!h-auto !whitespace-normal"
					>
						{{ navRoute.label }}
					</ButtonLink>
				</div>
			</div>
			<div
				class="nav-menu nav-menu-mobile"
				:class="{ expanded: isMobileMenuOpen }"
				@focusin="isMobileMenuOpen = true"
				@focusout="isMobileMenuOpen = false"
			>
				<div class="account-container">
					<ButtonLink
						v-if="auth.user"
						:to="`/user/${auth.user.username}`"
						class="account-button !h-auto !whitespace-normal"
					>
						<Avatar
							:src="auth.user.avatar_url"
							class="user-icon"
							:alt="formatMessage(messages.yourAvatarAlt)"
							aria-hidden="true"
							circle
						/>
						<div class="account-text">
							<div>@{{ auth.user.username }}</div>
							<div>{{ formatMessage(commonMessages.visitYourProfile) }}</div>
						</div>
					</ButtonLink>
					<ButtonLink
						v-else
						type="colored"
						color="brand"
						:to="signInRouteObj"
						class="!h-auto !whitespace-normal"
					>
						<LogInIcon aria-hidden="true" /> {{ formatMessage(commonMessages.signInButton) }}
					</ButtonLink>
				</div>
				<div class="links">
					<template v-if="auth.user">
						<Button
							type="colored"
							color="red"
							class="!h-auto !whitespace-normal"
							@click="logoutUser()"
						>
							<LogOutIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.signOutButton) }}
						</Button>
						<Button class="!h-auto !whitespace-normal" @click="$refs.modal_creation.show()">
							<PlusIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.createAProjectButton) }}
						</Button>
						<ButtonLink class="!h-auto !whitespace-normal" to="/dashboard/collections">
							<LibraryIcon class="icon" />
							{{ formatMessage(commonMessages.collectionsLabel) }}
						</ButtonLink>
						<ButtonLink class="!h-auto !whitespace-normal" to="/hosting/manage">
							<ServerIcon class="icon" />
							{{ formatMessage(commonMessages.serversLabel) }}
						</ButtonLink>
						<ButtonLink
							v-if="auth.user.role === 'moderator' || auth.user.role === 'admin'"
							class="!h-auto !whitespace-normal"
							to="/moderation"
						>
							<ScaleIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.moderationLabel) }}
						</ButtonLink>
						<ButtonLink
							v-if="flags.developerMode"
							class="!h-auto !whitespace-normal"
							to="/settings/flags"
						>
							<ToggleRightIcon aria-hidden="true" />
							{{ formatMessage(commonSettingsMessages.featureFlags) }}
						</ButtonLink>
					</template>
					<ButtonLink class="!h-auto !whitespace-normal" to="/settings">
						<SettingsIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.settingsLabel) }}
					</ButtonLink>
					<Button class="!h-auto !whitespace-normal" @click="changeTheme">
						<MoonIcon v-if="$theme.active === 'light'" class="icon" />
						<SunIcon v-else class="icon" />
						<span class="dropdown-item__text">
							{{ formatMessage(messages.changeTheme) }}
						</span>
					</Button>
				</div>
			</div>
			<div class="mobile-navbar" :class="{ expanded: isBrowseMenuOpen || isMobileMenuOpen }">
				<ButtonLink
					to="/"
					type="quiet"
					interaction="none"
					class="tab !h-auto !rounded-none !px-0"
					:title="formatMessage(navMenuMessages.home)"
					:aria-label="formatMessage(navMenuMessages.home)"
				>
					<HomeIcon aria-hidden="true" />
				</ButtonLink>
				<Button
					type="quiet"
					interaction="none"
					class="tab !h-auto !rounded-none !px-0"
					:class="{ 'router-link-exact-active': isBrowseMenuOpen }"
					:title="formatMessage(navMenuMessages.search)"
					:aria-label="formatMessage(navMenuMessages.search)"
					@click="toggleBrowseMenu()"
				>
					<template v-if="auth.user">
						<SearchIcon aria-hidden="true" />
					</template>
					<template v-else>
						<SearchIcon aria-hidden="true" class="smaller" />
						{{ formatMessage(navMenuMessages.search) }}
					</template>
				</Button>
				<template v-if="auth.user">
					<ButtonLink
						to="/dashboard/notifications"
						type="quiet"
						interaction="none"
						class="tab !h-auto !rounded-none !px-0"
						:aria-label="formatMessage(commonMessages.notificationsLabel)"
						:class="{
							'no-active': isMobileMenuOpen || isBrowseMenuOpen,
						}"
						:title="formatMessage(commonMessages.notificationsLabel)"
						@click="
							() => {
								isMobileMenuOpen = false
								isBrowseMenuOpen = false
							}
						"
					>
						<BellIcon aria-hidden="true" />
					</ButtonLink>
					<ButtonLink
						to="/dashboard"
						type="quiet"
						interaction="none"
						class="tab !h-auto !rounded-none !px-0"
						:aria-label="formatMessage(commonMessages.dashboardLabel)"
						:title="formatMessage(commonMessages.dashboardLabel)"
					>
						<ChartIcon aria-hidden="true" />
					</ButtonLink>
				</template>
				<Button
					type="quiet"
					interaction="none"
					class="tab !h-auto !rounded-none !px-0"
					:title="formatMessage(messages.toggleMenu)"
					:aria-label="
						isMobileMenuOpen ? formatMessage(messages.closeMenu) : formatMessage(messages.openMenu)
					"
					@click="toggleMobileMenu()"
				>
					<template v-if="!auth.user">
						<HamburgerIcon v-if="!isMobileMenuOpen" aria-hidden="true" />
						<XIcon v-else aria-hidden="true" />
					</template>
					<template v-else>
						<Avatar
							:src="auth.user.avatar_url"
							class="user-icon"
							:class="{ expanded: isMobileMenuOpen }"
							:alt="formatMessage(messages.yourAvatarAlt)"
							aria-hidden="true"
							circle
						/>
					</template>
				</Button>
			</div>
		</header>
		<main class="min-h-[calc(100vh-4.5rem-310.59px)]">
			<ProjectCreateModal v-if="auth.user" ref="modal_creation" />
			<CollectionCreateModal ref="modal_collection_creation" />
			<OrganizationCreateModal ref="modal_organization_creation" />
			<BatchCreditModal v-if="auth.user && isAdmin(auth.user)" ref="modal_batch_credit" />
			<slot id="main" />
		</main>
		<ModrinthFooter />
	</div>
</template>
<script setup>
import {
	AffiliateIcon,
	ArrowBigUpDashIcon,
	ArrowLeftRightIcon,
	BellIcon,
	BookOpenIcon,
	BoxIcon,
	BoxPlusIcon,
	BracesIcon,
	ChartIcon,
	CollectionPlusIcon,
	CompassIcon,
	CurrencyIcon,
	DownloadIcon,
	DropdownIcon,
	FileSearchCornerIcon,
	GlassesIcon,
	GlobeIcon,
	HamburgerIcon,
	HashIcon,
	HomeIcon,
	IssuesIcon,
	LibraryIcon,
	LogInIcon,
	LogOutIcon,
	MailIcon,
	ModrinthIcon,
	MoonIcon,
	OrganizationIcon,
	OrganizationPlusIcon,
	PackageOpenIcon,
	PaintbrushIcon,
	PlugIcon,
	PlusIcon,
	ReportIcon,
	ScaleIcon,
	SearchIcon,
	ServerIcon,
	ServerPlusIcon,
	ServerSearchIcon,
	ServerStackIcon,
	SettingsIcon,
	ShieldAlertIcon,
	SunIcon,
	ToggleRightIcon,
	TransferIcon,
	UserIcon,
	UserSearchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ButtonLink,
	commonMessages,
	commonProjectTypeCategoryMessages,
	commonSettingsMessages,
	createHostingIntercomIdentityKey,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectPageContext,
	injectUserPreferences,
	providePageContext,
	TeleportOverflowMenu,
	useHostingIntercom,
	UserRoleIcon,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, isStaff, UserBadge } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'

import { getTaxThreshold } from '@/providers/creator-withdraw.ts'
import TextLogo from '~/components/brand/TextLogo.vue'
import BatchCreditModal from '~/components/ui/admin/BatchCreditModal.vue'
import GeneratedStateErrorsBanner from '~/components/ui/banner/GeneratedStateErrorsBanner.vue'
import PreviewBanner from '~/components/ui/banner/PreviewBanner.vue'
import RussiaBanner from '~/components/ui/banner/RussiaBanner.vue'
import StagingBanner from '~/components/ui/banner/StagingBanner.vue'
import SubscriptionPaymentFailedBanner from '~/components/ui/banner/SubscriptionPaymentFailedBanner.vue'
import TaxComplianceBanner from '~/components/ui/banner/TaxComplianceBanner.vue'
import TaxIdMismatchBanner from '~/components/ui/banner/TaxIdMismatchBanner.vue'
import VerifyEmailBanner from '~/components/ui/banner/VerifyEmailBanner.vue'
import ViewOnModrinthBanner from '~/components/ui/banner/ViewOnModrinthBanner.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import OrganizationCreateModal from '~/components/ui/create/OrganizationCreateModal.vue'
import ProjectCreateModal from '~/components/ui/create/ProjectCreateModal.vue'
import ModrinthFooter from '~/components/ui/ModrinthFooter.vue'
import {
	forgetStoredAccount,
	switchToSignedOut,
	switchToStoredAccount,
	useStoredAccounts,
} from '~/composables/accounts.ts'
import { getAddAccountRouteObj, getSignInRouteObj } from '~/composables/auth.ts'
import { logout } from '~/composables/user.js'
import { errors as generatedStateErrors, taxComplianceThresholds } from '~/generated/state.json'
import { provideCurrentProjectId } from '~/providers/current-project.ts'
import { getProjectTypeMessage } from '~/utils/i18n-project-type.ts'
import { hasActiveMidas } from '~/utils/user-membership.ts'

const country = useUserCountry()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const { updatePreferences } = injectUserPreferences()

const auth = await useAuth()
const user = await useUser()

const cosmetics = useCosmetics()
const flags = useFeatureFlags()

const config = useRuntimeConfig()
const route = useNativeRoute()
const router = useNativeRouter()
const signInRouteObj = computed(() => getSignInRouteObj(route))
const addAccountRouteObj = computed(() => getAddAccountRouteObj(route))
const storedAccounts = useStoredAccounts()
const link = config.public.siteUrl + route.path.replace(/\/+$/, '')
const client = injectModrinthClient()
const pageContext = injectPageContext()
const hostingIntercomActive = computed(() => route.path.startsWith('/hosting') && !!auth.value.user)
const hostingIntercomServerId = computed(() => {
	const rawId = route.params.id
	return Array.isArray(rawId) ? rawId[0] : rawId
})
const hostingIntercom = useHostingIntercom({
	enabled: hostingIntercomActive,
	appId: computed(() => config.public.intercomAppId),
	fetchToken: fetchIntercomToken,
	identityKey: computed(() =>
		createHostingIntercomIdentityKey(auth.value.user, hostingIntercomServerId.value),
	),
})

providePageContext({
	...pageContext,
	intercomBubble: hostingIntercom.intercomBubble,
})

const { data: payoutBalance } = useQuery({
	queryKey: ['payout', 'balance'],
	queryFn: () => client.labrinth.payout_v3.getBalance(),
	enabled: computed(() => !!auth.value.user),
})

const showTaxComplianceBanner = computed(() => {
	if (flags.value.testTaxForm && auth.value.user) return true
	const bal = payoutBalance.value
	if (!bal) return false
	const threshold = getTaxThreshold(taxComplianceThresholds)
	const thresholdMet = (bal.withdrawn_ytd ?? 0) >= threshold
	const status = bal.form_completion_status ?? 'unknown'
	const isComplete = status === 'complete'
	const isTinMismatch = status === 'tin-mismatch'
	return !!auth.value.user && thresholdMet && !isComplete && !isTinMismatch
})

const showTinMismatchBanner = computed(() => {
	const bal = payoutBalance.value
	if (!bal) return false
	const status = bal.form_completion_status ?? 'unknown'
	return !!auth.value.user && status === 'tin-mismatch'
})

const PRIDE_COLLECTION_ID = 'M4c3ITvd'
const PRIDE_ARTICLE_SLUGS = ['pride-campaign-2025', 'pride-campaign-2026', 'proud-of-you-2026']
const PRIDE_CACHE_TIME = 1000 * 60 * 60 * 24

const { data: prideCollection } = useQuery({
	queryKey: computed(() => ['collection', PRIDE_COLLECTION_ID]),
	queryFn: () => client.labrinth.collections.get(PRIDE_COLLECTION_ID),
	staleTime: PRIDE_CACHE_TIME,
	gcTime: PRIDE_CACHE_TIME,
})

const prideProjectIds = computed(() => new Set(prideCollection.value?.projects ?? []))

const currentProjectId = ref()
provideCurrentProjectId(currentProjectId)

const showPrideBackdrop = computed(() => {
	if (PRIDE_ARTICLE_SLUGS.includes(route.params.slug)) {
		return true
	}
	if (route.params.collection === PRIDE_COLLECTION_ID) {
		return true
	}
	return !!currentProjectId.value && prideProjectIds.value.has(currentProjectId.value)
})

async function fetchIntercomToken() {
	return $fetch('/api/intercom/messenger-jwt', {
		query: hostingIntercomServerId.value ? { server_id: hostingIntercomServerId.value } : {},
	})
}

function requireVerifiedEmail(action) {
	if (!auth.value.user?.email_verified) {
		addNotification({
			title: formatMessage(messages.emailVerificationRequired),
			text: formatMessage(messages.verifyEmailBeforePublishing),
			type: 'error',
		})
		return
	}

	action()
}

const navMenuMessages = defineMessages({
	home: {
		id: 'layout.nav.home',
		defaultMessage: 'Home',
	},
	search: {
		id: 'layout.nav.search',
		defaultMessage: 'Search',
	},
	discover: {
		id: 'layout.nav.discover',
		defaultMessage: 'Discover',
	},
	hostAServer: {
		id: 'layout.nav.host-a-server',
		defaultMessage: 'Host a server',
	},
	getModrinthApp: {
		id: 'layout.nav.get-modrinth-app',
		defaultMessage: 'Get Modrinth App',
	},
	modrinthApp: {
		id: 'layout.nav.modrinth-app',
		defaultMessage: 'Modrinth App',
	},
})

const messages = defineMessages({
	toggleMenu: {
		id: 'layout.menu-toggle.action',
		defaultMessage: 'Toggle menu',
	},
	yourAvatarAlt: {
		id: 'layout.avatar.alt',
		defaultMessage: 'Your avatar',
	},
	changeTheme: {
		id: 'layout.action.change-theme',
		defaultMessage: 'Change theme',
	},
	modrinthHomePage: {
		id: 'layout.nav.modrinth-home-page',
		defaultMessage: 'Modrinth home page',
	},
	createNew: {
		id: 'layout.action.create-new',
		defaultMessage: 'Create new...',
	},
	publish: {
		id: 'layout.action.publish',
		defaultMessage: 'Publish',
	},
	emailVerificationRequired: {
		id: 'layout.publish.email-verification-required.title',
		defaultMessage: 'Email verification required',
	},
	verifyEmailBeforePublishing: {
		id: 'layout.publish.email-verification-required.description',
		defaultMessage: 'You must verify your email before publishing on Modrinth.',
	},
	reviewProjects: {
		id: 'layout.action.review-projects',
		defaultMessage: 'Project review',
	},
	techReview: {
		id: 'layout.action.tech-review',
		defaultMessage: 'Tech review',
	},
	reports: {
		id: 'layout.action.reports',
		defaultMessage: 'Review reports',
	},
	externalProjects: {
		id: 'layout.action.external-projects',
		defaultMessage: 'External projects',
	},
	userLookup: {
		id: 'layout.action.user-lookup',
		defaultMessage: 'User lookup',
	},
	fileLookup: {
		id: 'layout.action.file-lookup',
		defaultMessage: 'File lookup',
	},
	manageServerNotices: {
		id: 'layout.action.manage-server-notices',
		defaultMessage: 'Manage server notices',
	},
	manageAffiliates: {
		id: 'layout.action.manage-affiliates',
		defaultMessage: 'Manage affiliate links',
	},
	analyticsEvents: {
		id: 'layout.action.analytics-events',
		defaultMessage: 'Analytics events',
	},
	newProject: {
		id: 'layout.action.new-project',
		defaultMessage: 'New project',
	},
	newServerProject: {
		id: 'layout.action.new-server-project',
		defaultMessage: 'New server project',
	},
	newCollection: {
		id: 'layout.action.new-collection',
		defaultMessage: 'New collection',
	},
	newOrganization: {
		id: 'layout.action.new-organization',
		defaultMessage: 'New organization',
	},
	profile: {
		id: 'layout.nav.profile',
		defaultMessage: 'Profile',
	},
	savedProjects: {
		id: 'layout.nav.saved-projects',
		defaultMessage: 'Saved projects',
	},
	upgradeToModrinthPlus: {
		id: 'layout.nav.upgrade-to-modrinth-plus',
		defaultMessage: 'Upgrade to Modrinth+',
	},
	projects: {
		id: 'layout.nav.projects',
		defaultMessage: 'Projects',
	},
	organizations: {
		id: 'layout.nav.organizations',
		defaultMessage: 'Organizations',
	},
	revenue: {
		id: 'layout.nav.revenue',
		defaultMessage: 'Revenue',
	},
	activeReports: {
		id: 'layout.nav.active-reports',
		defaultMessage: 'Active reports',
	},
	myServers: {
		id: 'layout.nav.my-servers',
		defaultMessage: 'My servers',
	},
	switchAccount: {
		id: 'layout.nav.switch-account',
		defaultMessage: 'Switch account',
	},
	addAccount: {
		id: 'layout.nav.add-account',
		defaultMessage: 'Add account',
	},
	removeAccount: {
		id: 'layout.nav.remove-account',
		defaultMessage: 'Remove account',
	},
	accountSwitchFailed: {
		id: 'layout.nav.switch-account-failed',
		defaultMessage: "Couldn't switch accounts. Please try again.",
	},
	openMenu: {
		id: 'layout.mobile.open-menu',
		defaultMessage: 'Open menu',
	},
	closeMenu: {
		id: 'layout.mobile.close-menu',
		defaultMessage: 'Close menu',
	},
})

useFavicon()
useHead({
	link: [
		{
			rel: 'canonical',
			href: link,
		},
	],
})
useSeoMeta({
	title: 'Modrinth',
	description: () =>
		formatMessage({
			id: 'layout.meta.description',
			defaultMessage:
				'Download Minecraft mods, plugins, datapacks, shaders, resourcepacks, and modpacks on Modrinth. ' +
				'Discover and publish projects on Modrinth with a modern, easy to use interface and API.',
		}),
	publisher: 'Modrinth',
	themeColor: '#1bd96a',
	colorScheme: 'dark light',

	// OpenGraph
	ogTitle: 'Modrinth',
	ogSiteName: 'Modrinth',
	ogDescription: () =>
		formatMessage({
			id: 'layout.meta.og-description',
			defaultMessage: 'Discover and publish Minecraft content!',
		}),
	ogType: 'website',
	ogImage: 'https://cdn.modrinth.com/modrinth-new.png',
	ogUrl: link,

	// Twitter
	twitterCard: 'summary',
	twitterSite: '@modrinth',
})

const isMobileMenuOpen = ref(false)
const isBrowseMenuOpen = ref(false)
const navRoutes = computed(() => [
	{
		id: 'mods',
		label: formatMessage(getProjectTypeMessage('mod', true)),
		href: '/discover/mods',
	},
	{
		label: formatMessage(getProjectTypeMessage('resourcepack', true)),
		href: '/discover/resourcepacks',
	},
	{
		label: formatMessage(getProjectTypeMessage('datapack', true)),
		href: '/discover/datapacks',
	},
	{
		label: formatMessage(getProjectTypeMessage('shader', true)),
		href: '/discover/shaders',
	},
	{
		label: formatMessage(getProjectTypeMessage('modpack', true)),
		href: '/discover/modpacks',
	},
	{
		label: formatMessage(getProjectTypeMessage('plugin', true)),
		href: '/discover/plugins',
	},
	{
		label: formatMessage(getProjectTypeMessage('server', true)),
		href: '/discover/servers',
	},
])

const accountSwitcherAccounts = computed(() =>
	storedAccounts.value.map((account) => ({
		...account,
		optionId: `account-${account.id}`,
		current: account.id === auth.value.user?.id,
	})),
)

const accountSwitcherOptions = computed(() => [
	...accountSwitcherAccounts.value.map((account) => ({
		id: account.optionId,
		label: account.username,
		selected: account.current,
		action: () => switchAccount(account),
		trailingAction: {
			label: formatMessage(messages.removeAccount),
			icon: XIcon,
			color: 'red',
			action: () => removeAccount(account),
		},
	})),
	{
		type: 'divider',
	},
	{
		id: 'add-account',
		label: formatMessage(messages.addAccount),
		icon: PlusIcon,
		type: 'link',
		to: addAccountRouteObj.value,
	},
])

async function switchAccount(account) {
	if (account.current) return

	const result = await switchToStoredAccount(account)
	if (result === 'error') {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: formatMessage(messages.accountSwitchFailed),
			type: 'error',
		})
	}
}

async function removeAccount(account) {
	if (account.current) {
		await logout()
		return
	}

	forgetStoredAccount(account.id)
}

const userMenuOptions = computed(() => {
	const user = auth.value.user
	if (!user) return []

	let options = [
		{
			id: 'profile',
			label: formatMessage(messages.profile),
			icon: UserIcon,
			type: 'link',
			to: `/user/${user.username}`,
		},
		{
			id: 'plus',
			label: formatMessage(messages.upgradeToModrinthPlus),
			icon: ArrowBigUpDashIcon,
			type: 'link',
			to: '/plus',
			tone: 'purple',
			shown: !flags.value.hidePlusPromoInUserMenu && !hasActiveMidas(user),
		},
		{
			id: 'servers',
			label: formatMessage(messages.myServers),
			icon: ServerStackIcon,
			type: 'link',
			to: '/hosting/manage',
		},
		{
			id: 'flags',
			label: formatMessage(commonSettingsMessages.featureFlags),
			icon: ToggleRightIcon,
			type: 'link',
			to: '/settings/flags',
			shown: flags.value.developerMode,
		},
		{
			id: 'settings',
			label: formatMessage(commonMessages.settingsLabel),
			icon: SettingsIcon,
			type: 'link',
			to: '/settings',
		},
	]

	// TODO: Only show if user has projects
	options = [
		...options,
		{
			type: 'divider',
		},
		{
			id: 'notifications',
			label: formatMessage(commonMessages.notificationsLabel),
			icon: BellIcon,
			type: 'link',
			to: '/dashboard/notifications',
		},
		{
			id: 'reports',
			label: formatMessage(messages.activeReports),
			icon: ReportIcon,
			type: 'link',
			to: '/dashboard/reports',
		},
		{
			id: 'saved',
			label: formatMessage(commonMessages.collectionsLabel),
			icon: LibraryIcon,
			type: 'link',
			to: '/dashboard/collections',
		},
		{
			type: 'divider',
		},
		{
			id: 'projects',
			label: formatMessage(messages.projects),
			icon: BoxIcon,
			type: 'link',
			to: '/dashboard/projects',
		},
		{
			id: 'organizations',
			label: formatMessage(messages.organizations),
			icon: OrganizationIcon,
			type: 'link',
			to: '/dashboard/organizations',
		},
		{
			id: 'analytics',
			label: formatMessage(commonMessages.analyticsButton),
			icon: ChartIcon,
			type: 'link',
			to: '/dashboard/analytics',
		},
		{
			id: 'affiliate-links',
			label: formatMessage(commonMessages.affiliateLinksButton),
			icon: AffiliateIcon,
			type: 'link',
			to: '/dashboard/affiliate-links',
			shown: Boolean(user.badges & UserBadge.AFFILIATE),
		},
		{
			id: 'revenue',
			label: formatMessage(messages.revenue),
			icon: CurrencyIcon,
			type: 'link',
			to: '/dashboard/revenue',
		},
	]

	options = [
		...options,
		{
			type: 'divider',
		},
		{
			id: 'switch-account',
			label: formatMessage(messages.switchAccount),
			icon: ArrowLeftRightIcon,
			type: 'submenu',
			options: accountSwitcherOptions.value,
		},
		{
			id: 'sign-out',
			label: formatMessage(commonMessages.signOutButton),
			icon: LogOutIcon,
			tone: 'red',
			hoverFilled: true,
			action: () => logoutUser(),
		},
	]
	return options
})

const isDiscovering = computed(
	() => route.name && route.name.startsWith('discover-') && !route.query.sid,
)

const isDiscoveringSubpage = computed(
	() => route.name && route.name.startsWith('type-project') && !route.query.sid,
)

const isRussia = computed(() => country.value === 'ru')

const rCount = ref(0)

const randomProjects = ref([])
const disableRandomProjects = ref(false)

const disableRandomProjectsForRoute = computed(
	() =>
		route.name.startsWith('hosting') ||
		route.name.includes('settings') ||
		route.name.includes('admin'),
)

async function onKeyDown(event) {
	if (disableRandomProjects.value || disableRandomProjectsForRoute.value) {
		return
	}

	if (event.key === 'r') {
		rCount.value++

		if (randomProjects.value.length < 3) {
			randomProjects.value = await client.labrinth.projects_v2.getRandom(50).catch((err) => {
				console.error(err)
				return []
			})
		}
	}

	if (rCount.value >= 40) {
		rCount.value = 0
		const randomProject = randomProjects.value[0]
		await router.push(`/project/${randomProject.slug}`)
		randomProjects.value.splice(0, 1)
	}
}

function onKeyUp(event) {
	if (event.key === 'r') {
		rCount.value = 0
	}
}

onMounted(() => {
	if (window && import.meta.client) {
		window.history.scrollRestoration = 'auto'
	}

	runAnalytics()

	window.addEventListener('keydown', onKeyDown)
	window.addEventListener('keyup', onKeyUp)
})

watch(
	() => route.path,
	() => {
		isMobileMenuOpen.value = false
		isBrowseMenuOpen.value = false

		if (import.meta.client) {
			document.body.style.overflowY = 'scroll'
			document.body.setAttribute('tabindex', '-1')
			document.body.removeAttribute('tabindex')
		}

		runAnalytics()
	},
)

async function logoutUser() {
	await switchToSignedOut()
}

function runAnalytics() {
	if (import.meta.dev) {
		return
	}

	const config = useRuntimeConfig()
	const replacedUrl = config.public.apiBaseUrl.replace('v2/', '')

	try {
		setTimeout(() => {
			$fetch(`${replacedUrl}analytics/view`, {
				method: 'POST',
				body: {
					url: window.location.href,
				},
				headers: {
					Authorization: auth.value.token,
				},
			})
				.then(() => {})
				.catch(() => {})
		})
	} catch (e) {
		console.error(`Sending analytics failed (CORS error? If so, ignore)`, e)
	}
}
function toggleMobileMenu() {
	isMobileMenuOpen.value = !isMobileMenuOpen.value
	if (isMobileMenuOpen.value) {
		isBrowseMenuOpen.value = false
	}
}
function toggleBrowseMenu() {
	isBrowseMenuOpen.value = !isBrowseMenuOpen.value

	if (isBrowseMenuOpen.value) {
		isMobileMenuOpen.value = false
	}
}

const theme = useTheme()

function changeTheme() {
	const selectedTheme = theme.cycle()
	if (!theme.syncAcrossDevices) return

	void updatePreferences({
		appearance: {
			auto: false,
			theme: selectedTheme,
		},
	}).catch(() => undefined)
}
</script>

<style lang="scss">
@import '~/assets/styles/global.scss';
// @import '@modrinth/assets';

.layout {
	min-height: 100vh;
	display: block;

	@media screen and (min-width: 1024px) {
		min-height: calc(100vh - var(--spacing-card-bg));
	}

	main {
		grid-area: main;
	}
}

@media (min-width: 1024px) {
	.layout {
		main {
			.alpha-alert {
				margin: 1rem;

				.wrapper {
					padding: 1rem 2rem 1rem 1rem;
				}
			}
		}
	}
}

@media (max-width: 1200px) {
	.app-btn {
		display: none;
	}
}

.mobile-navigation {
	display: none;

	.nav-menu {
		width: 100%;
		position: fixed;
		bottom: calc(var(--size-mobile-navbar-height) - var(--size-rounded-card));
		padding-bottom: var(--size-rounded-card);
		left: 0;
		background-color: var(--color-raised-bg);
		z-index: 11; // 20 = modals, 10 = svg icons
		transform: translateY(calc(100% + env(safe-area-inset-bottom)));
		transition: transform 0.4s cubic-bezier(0.54, 0.84, 0.42, 1);
		border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;
		box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0);

		.links,
		.account-container {
			display: grid;
			grid-template-columns: repeat(1, 1fr);
			grid-gap: 1rem;
			justify-content: center;
			padding: 1rem;

			> button,
			> a {
				width: 100%;
				max-width: 500px;
				padding: 0.75rem;
				justify-content: center;
				font-weight: 600;
				font-size: 1rem;
				margin: 0 auto;
			}
		}

		.cascade-links {
			@media screen and (min-width: 354px) {
				grid-template-columns: repeat(2, 1fr);
			}

			@media screen and (min-width: 674px) {
				grid-template-columns: repeat(3, 1fr);
			}
		}

		&-browse {
			&.expanded {
				transform: translateY(0);
				box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
			}
		}

		&-mobile {
			.account-container {
				opacity: 0;
				padding-bottom: 0;
				pointer-events: none;
				transition: opacity 0.15s ease-in-out;
				visibility: hidden;

				.account-button {
					padding: var(--spacing-card-md);
					display: flex;
					align-items: center;
					justify-content: center;
					gap: 0.5rem;

					.user-icon {
						width: 2.25rem;
						height: 2.25rem;
					}

					.account-text {
						flex-grow: 0;
					}
				}
			}

			&.expanded {
				transform: translateY(0);
				box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);

				.account-container {
					opacity: 1;
					pointer-events: auto;
					visibility: visible;
				}
			}
		}
	}

	.mobile-navbar {
		display: flex;
		height: calc(var(--size-mobile-navbar-height) + env(safe-area-inset-bottom));
		border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;
		padding-bottom: env(safe-area-inset-bottom);
		position: fixed;
		left: 0;
		bottom: 0;
		background-color: var(--color-raised-bg);
		box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
		z-index: 11; // 20 = modals, 10 = svg icons
		width: 100%;
		align-items: center;
		justify-content: space-between;
		transition: border-radius 0.3s ease-out;
		border-top: 2px solid rgba(0, 0, 0, 0);
		box-sizing: border-box;

		&::after {
			content: '';
			position: absolute;
			bottom: 2px;
			left: 0;
			width: 100%;
			height: 300px;
			background-color: var(--color-raised-bg);
			transform: translateY(100%);
		}

		&.expanded {
			box-shadow: none;
			border-radius: 0;
		}

		.tab {
			position: relative;
			background: none;
			display: flex;
			flex-basis: 0;
			justify-content: center;
			align-items: center;
			flex-direction: row;
			gap: 0.25rem;
			font-weight: bold;
			padding: 0;
			transition: color ease-in-out 0.15s;
			color: var(--color-text-inactive);
			text-align: center;

			&.browse {
				svg {
					transform: rotate(180deg);
					transition: transform ease-in-out 0.3s;

					&.closed {
						transform: rotate(0deg);
					}
				}
			}

			&.bubble {
				&::after {
					background-color: var(--color-brand);
					border-radius: var(--size-rounded-max);
					content: '';
					height: 0.5rem;
					position: absolute;
					left: 1.5rem;
					top: 0;
					width: 0.5rem;
				}
			}

			svg {
				height: 1.75rem;
				width: 1.75rem;

				&.smaller {
					width: 1.25rem;
					height: 1.25rem;
				}
			}

			.user-icon {
				width: 2rem;
				height: 2rem;
				transition: border ease-in-out 0.15s;
				border: 0 solid var(--color-brand);
				box-sizing: border-box;

				&.expanded {
					border: 2px solid var(--color-brand);
				}
			}

			&:hover,
			&:focus {
				color: var(--color-text);
			}

			&:first-child {
				margin-left: 2rem;
			}

			&:last-child {
				margin-right: 2rem;
			}

			&.router-link-exact-active:not(&.no-active) {
				svg {
					color: var(--color-brand);
				}

				color: var(--color-brand);
			}
		}
	}
}

@media (pointer: coarse) and (max-width: 640px) {
	.desktop-only {
		display: none;
	}
}

@media (pointer: coarse) and (max-width: 640px) {
	.mobile-navigation {
		display: flex;
	}
}

.over-the-top-random-animation {
	position: fixed;
	z-index: 100;
	inset: 0;
	display: flex;
	justify-content: center;
	align-items: center;
	pointer-events: none;
	scale: 0.5;
	transition: all 0.5s ease-out;
	opacity: 0;
	animation:
		tilt-shaking calc(0.2s / (max((var(--_r-count) - 20), 1) / 20)) linear infinite,
		translate-x-shaking calc(0.3s / (max((var(--_r-count) - 20), 1) / 20)) linear infinite,
		translate-y-shaking calc(0.25s / (max((var(--_r-count) - 20), 1) / 20)) linear infinite;

	&.threshold {
		opacity: 1;
	}

	&.rings-expand {
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
			scale: calc(1 + max((var(--_r-count) - 20), 0) * 0.1);
			transition: all 0.2s ease-out;
			width: 20rem;
			height: 20rem;
		}
	}
}

@keyframes tilt-shaking {
	0% {
		rotate: 0deg;
	}

	25% {
		rotate: calc(1deg * (var(--_r-count) - 20));
	}

	50% {
		rotate: 0deg;
	}

	75% {
		rotate: calc(-1deg * (var(--_r-count) - 20));
	}

	100% {
		rotate: 0deg;
	}
}

@keyframes translate-x-shaking {
	0% {
		translate: 0;
	}

	25% {
		translate: calc(2px * (var(--_r-count) - 20));
	}

	50% {
		translate: 0;
	}

	75% {
		translate: calc(-2px * (var(--_r-count) - 20));
	}

	100% {
		translate: 0;
	}
}

@keyframes translate-y-shaking {
	0% {
		transform: translateY(0);
	}

	25% {
		transform: translateY(calc(2px * (var(--_r-count) - 20)));
	}

	50% {
		transform: translateY(0);
	}

	75% {
		transform: translateY(calc(-2px * (var(--_r-count) - 20)));
	}

	100% {
		transform: translateY(0);
	}
}

.pride-backdrop {
	background-image: linear-gradient(to right, #c20732, #f57203, #ffd632, #21ca8b, #2f9ff2, #e420fc);
	mask-image: linear-gradient(to bottom, rgba(0, 0, 0, 1), rgba(0, 0, 0, 0) 80%);
	height: 30rem;
	opacity: 0;
	transition: opacity 1s ease;
}

.pride-backdrop.shown {
	opacity: 0.08;
}

.light-mode .pride-backdrop.shown,
.light .pride-backdrop.shown {
	opacity: 0.15;
}
</style>
