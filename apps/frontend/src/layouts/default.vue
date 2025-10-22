<template>
	<div class="pointer-events-none fixed inset-0 z-[-1]">
		<div id="fixed-background-teleport" class="relative"></div>
	</div>
	<div class="pointer-events-none absolute inset-0 z-[-1]">
		<div id="absolute-background-teleport" class="relative"></div>
	</div>
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
	<div ref="main_page" class="layout" :class="{ 'expanded-mobile-nav': isBrowseMenuOpen }">
		<PagewideBanner v-if="isRussia && !flags.hideRussiaCensorshipBanner" variant="error">
			<template #title>
				<div class="flex flex-col gap-1 text-contrast">
					<span lang="ru">К сожалению, Modrinth скоро станет недоступен в России</span>
					<span class="text-sm font-medium opacity-50" lang="en">
						Modrinth will soon be unavailable in Russia
					</span>
				</div>
			</template>
			<template #description>
				<p class="m-0" lang="ru">
					Российское правительство потребовало от нас заблокировать некоторые проекты на Modrinth,
					но мы решили отказать им в цензуре.
				</p>
				<p class="-mt-2 mb-0 text-sm opacity-50" lang="en">
					The Russian government has asked us to censor certain topics on Modrinth and we have
					decided to refuse to comply with their requests.
				</p>

				<p class="m-0 font-semibold" lang="ru">
					Пожалуйста, найдите какой-нибудь надёжный VPN или прокси, чтобы не потерять доступ к
					Modrinth.
				</p>
				<p class="-mt-2 mb-0 text-sm opacity-50" lang="en">
					Please seek a reputable VPN or proxy of some kind to continue to access Modrinth in
					Russia.
				</p>
			</template>
			<template #actions>
				<div class="mt-2 flex w-fit gap-2">
					<ButtonStyled color="brand">
						<nuxt-link to="/news/article/standing-by-our-values-russian">
							<BookTextIcon /> Прочесть наше полное заявление
							<span class="text-xs font-medium">(Перевод на русский)</span>
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled>
						<nuxt-link to="/news/article/standing-by-our-values">
							<BookTextIcon /> Read our full statement
							<span class="text-xs font-medium">(English)</span>
						</nuxt-link>
					</ButtonStyled>
				</div>
			</template>
			<template #actions_right>
				<ButtonStyled circular type="transparent">
					<button
						v-tooltip="formatMessage(commonMessages.closeButton)"
						@click="hideRussiaCensorshipBanner"
					>
						<XIcon :aria-label="formatMessage(commonMessages.closeButton)" />
					</button>
				</ButtonStyled>
			</template>
		</PagewideBanner>
		<PagewideBanner v-if="showTinMismatchBanner" variant="error">
			<template #title>
				<span>{{ formatMessage(tinMismatchBannerMessages.title) }}</span>
			</template>
			<template #description>
				<span>{{ formatMessage(tinMismatchBannerMessages.description) }}</span>
			</template>
			<template #actions>
				<div class="flex w-fit flex-row">
					<ButtonStyled color="red">
						<nuxt-link to="https://support.modrinth.com" target="_blank" rel="noopener">
							<MessageIcon /> {{ formatMessage(tinMismatchBannerMessages.action) }}
						</nuxt-link>
					</ButtonStyled>
				</div>
			</template>
		</PagewideBanner>
		<PagewideBanner v-if="showTaxComplianceBanner" variant="warning">
			<template #title>
				<span>{{ formatMessage(taxBannerMessages.title) }}</span>
			</template>
			<template #description>
				<span>{{ formatMessage(taxBannerMessages.description) }}</span>
			</template>
			<template #actions>
				<ButtonStyled color="orange">
					<button @click="openTaxForm">
						<FileTextIcon /> {{ formatMessage(taxBannerMessages.action) }}
					</button>
				</ButtonStyled>
			</template>
		</PagewideBanner>
		<PagewideBanner
			v-if="auth.user && !auth.user.email_verified && route.path !== '/auth/verify-email'"
			variant="warning"
		>
			<template #title>
				<span>
					{{
						auth?.user?.email
							? formatMessage(verifyEmailBannerMessages.title)
							: formatMessage(addEmailBannerMessages.title)
					}}
				</span>
			</template>
			<template #description>
				<span>
					{{
						auth?.user?.email
							? formatMessage(verifyEmailBannerMessages.description)
							: formatMessage(addEmailBannerMessages.description)
					}}
				</span>
			</template>
			<template #actions>
				<button v-if="auth?.user?.email" class="btn" @click="handleResendEmailVerification">
					{{ formatMessage(verifyEmailBannerMessages.action) }}
				</button>
				<nuxt-link v-else class="btn" to="/settings/account">
					<SettingsIcon aria-hidden="true" />
					{{ formatMessage(addEmailBannerMessages.action) }}
				</nuxt-link>
			</template>
		</PagewideBanner>
		<PagewideBanner
			v-if="
				user.subscriptions.some((x) => x.status === 'payment-failed') &&
				route.path !== '/settings/billing'
			"
			variant="error"
		>
			<template #title>
				<span>{{ formatMessage(subscriptionPaymentFailedBannerMessages.title) }}</span>
			</template>
			<template #description>
				<span>{{ formatMessage(subscriptionPaymentFailedBannerMessages.description) }}</span>
			</template>
			<template #actions>
				<nuxt-link class="btn" to="/settings/billing">
					<SettingsIcon aria-hidden="true" />
					{{ formatMessage(subscriptionPaymentFailedBannerMessages.action) }}
				</nuxt-link>
			</template>
		</PagewideBanner>
		<PagewideBanner
			v-if="
				config.public.apiBaseUrl.startsWith('https://staging-api.modrinth.com') &&
				!cosmetics.hideStagingBanner
			"
			variant="warning"
		>
			<template #title>
				<span>{{ formatMessage(stagingBannerMessages.title) }}</span>
			</template>
			<template #description>
				{{ formatMessage(stagingBannerMessages.description) }}
			</template>
			<template #actions_right>
				<Button
					transparent
					icon-only
					:aria-label="formatMessage(commonMessages.closeButton)"
					@click="hideStagingBanner"
				>
					<XIcon aria-hidden="true" />
				</Button>
			</template>
		</PagewideBanner>
		<PagewideBanner v-if="generatedStateErrors?.length" variant="error">
			<template #title>
				<span>{{ formatMessage(failedToBuildBannerMessages.title) }}</span>
			</template>
			<template #description>
				{{
					formatMessage(failedToBuildBannerMessages.description, {
						errors: generatedStateErrors,
						url: config.public.apiBaseUrl,
					})
				}}
			</template>
		</PagewideBanner>

		<CreatorTaxFormModal
			ref="taxFormModalRef"
			close-button-text="Close"
			:emit-success-on-close="false"
		/>
		<header
			class="experimental-styles-within desktop-only relative z-[5] mx-auto grid max-w-[1280px] grid-cols-[1fr_auto] items-center gap-2 px-6 py-4 lg:grid-cols-[auto_1fr_auto]"
		>
			<div>
				<NuxtLink to="/" :aria-label="formatMessage(messages.modrinthHomePage)">
					<TextLogo aria-hidden="true" class="h-7 w-auto text-contrast" />
				</NuxtLink>
			</div>
			<div
				:class="`col-span-2 row-start-2 flex flex-wrap justify-center ${flags.projectTypesPrimaryNav ? 'gap-2' : 'gap-4'} lg:col-span-1 lg:row-start-auto`"
			>
				<template v-if="flags.projectTypesPrimaryNav">
					<ButtonStyled
						type="transparent"
						:highlighted="route.name === 'search-mods' || route.path.startsWith('/mod/')"
						:highlighted-style="
							route.name === 'search-mods' ? 'main-nav-primary' : 'main-nav-secondary'
						"
					>
						<nuxt-link to="/mods">
							<BoxIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.mod) }}
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled
						type="transparent"
						:highlighted="
							route.name === 'search-resourcepacks' || route.path.startsWith('/resourcepack/')
						"
						:highlighted-style="
							route.name === 'search-resourcepacks' ? 'main-nav-primary' : 'main-nav-secondary'
						"
					>
						<nuxt-link to="/resourcepacks">
							<PaintbrushIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.resourcepack) }}
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled
						type="transparent"
						:highlighted="route.name === 'search-datapacks' || route.path.startsWith('/datapack/')"
						:highlighted-style="
							route.name === 'search-datapacks' ? 'main-nav-primary' : 'main-nav-secondary'
						"
					>
						<nuxt-link to="/datapacks">
							<BracesIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.datapack) }}
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled
						type="transparent"
						:highlighted="route.name === 'search-modpacks' || route.path.startsWith('/modpack/')"
						:highlighted-style="
							route.name === 'search-modpacks' ? 'main-nav-primary' : 'main-nav-secondary'
						"
					>
						<nuxt-link to="/modpacks">
							<PackageOpenIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.modpack) }}
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled
						type="transparent"
						:highlighted="route.name === 'search-shaders' || route.path.startsWith('/shader/')"
						:highlighted-style="
							route.name === 'search-shaders' ? 'main-nav-primary' : 'main-nav-secondary'
						"
					>
						<nuxt-link to="/shaders">
							<GlassesIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.shader) }}
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled
						type="transparent"
						:highlighted="route.name === 'search-plugins' || route.path.startsWith('/plugin/')"
						:highlighted-style="
							route.name === 'search-plugins' ? 'main-nav-primary' : 'main-nav-secondary'
						"
					>
						<nuxt-link to="/plugins">
							<PlugIcon aria-hidden="true" />
							{{ formatMessage(commonProjectTypeCategoryMessages.plugin) }}
						</nuxt-link>
					</ButtonStyled>
				</template>
				<template v-else>
					<ButtonStyled
						type="transparent"
						:highlighted="isDiscovering || isDiscoveringSubpage"
						:highlighted-style="isDiscoveringSubpage ? 'main-nav-secondary' : 'main-nav-primary'"
					>
						<TeleportOverflowMenu
							:options="[
								{
									id: 'mods',
									action: '/mods',
								},
								{
									id: 'resourcepacks',
									action: '/resourcepacks',
								},
								{
									id: 'datapacks',
									action: '/datapacks',
								},
								{
									id: 'shaders',
									action: '/shaders',
								},
								{
									id: 'modpacks',
									action: '/modpacks',
								},
								{
									id: 'plugins',
									action: '/plugins',
								},
							]"
							hoverable
						>
							<BoxIcon
								v-if="route.name === 'search-mods' || route.path.startsWith('/mod/')"
								aria-hidden="true"
							/>
							<PaintbrushIcon
								v-else-if="
									route.name === 'search-resourcepacks' || route.path.startsWith('/resourcepack/')
								"
								aria-hidden="true"
							/>
							<BracesIcon
								v-else-if="route.name === 'search-datapacks' || route.path.startsWith('/datapack/')"
								aria-hidden="true"
							/>
							<PackageOpenIcon
								v-else-if="route.name === 'search-modpacks' || route.path.startsWith('/modpack/')"
								aria-hidden="true"
							/>
							<GlassesIcon
								v-else-if="route.name === 'search-shaders' || route.path.startsWith('/shader/')"
								aria-hidden="true"
							/>
							<PlugIcon
								v-else-if="route.name === 'search-plugins' || route.path.startsWith('/plugin/')"
								aria-hidden="true"
							/>
							<CompassIcon v-else aria-hidden="true" />
							<span class="hidden md:contents">{{
								formatMessage(navMenuMessages.discoverContent)
							}}</span>
							<span class="contents md:hidden">{{ formatMessage(navMenuMessages.discover) }}</span>
							<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />

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
						</TeleportOverflowMenu>
					</ButtonStyled>
					<ButtonStyled
						type="transparent"
						:highlighted="
							route.name?.startsWith('servers') ||
							(route.name?.startsWith('search-') && route.query.sid)
						"
						:highlighted-style="
							route.name === 'servers' ? 'main-nav-primary' : 'main-nav-secondary'
						"
					>
						<nuxt-link to="/servers">
							<ServerIcon aria-hidden="true" />
							{{ formatMessage(navMenuMessages.hostAServer) }}
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled type="transparent" :highlighted="route.name === 'app'">
						<nuxt-link to="/app">
							<DownloadIcon aria-hidden="true" />
							<span class="hidden md:contents">{{
								formatMessage(navMenuMessages.getModrinthApp)
							}}</span>
							<span class="contents md:hidden">{{
								formatMessage(navMenuMessages.modrinthApp)
							}}</span>
						</nuxt-link>
					</ButtonStyled>
				</template>
			</div>
			<div class="flex items-center gap-1">
				<ButtonStyled type="transparent">
					<OverflowMenu
						v-if="auth.user && isStaff(auth.user)"
						class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
						position="bottom"
						direction="left"
						:dropdown-id="`${basePopoutId}-staff`"
						:aria-label="formatMessage(messages.createNew)"
						:options="[
							{
								id: 'review-projects',
								color: 'orange',
								link: '/moderation/',
							},
							{
								id: 'review-reports',
								color: 'orange',
								link: '/moderation/reports',
							},
							{
								divider: true,
							},
							{
								id: 'file-lookup',
								link: '/admin/file_lookup',
							},
							{
								divider: true,
								shown: isAdmin(auth.user),
							},
							{
								id: 'user-lookup',
								color: 'primary',
								link: '/admin/user_email',
								shown: isAdmin(auth.user),
							},
							{
								id: 'servers-notices',
								color: 'primary',
								link: '/admin/servers/notices',
								shown: isAdmin(auth.user),
							},
							{
								id: 'servers-nodes',
								color: 'primary',
								link: '/admin/servers/nodes',
								shown: isAdmin(auth.user),
							},
						]"
					>
						<ModrinthIcon aria-hidden="true" />
						<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
						<template #review-projects>
							<ScaleIcon aria-hidden="true" /> {{ formatMessage(messages.reviewProjects) }}
						</template>
						<template #review-reports>
							<ReportIcon aria-hidden="true" /> {{ formatMessage(messages.reports) }}
						</template>
						<template #user-lookup>
							<UserIcon aria-hidden="true" /> {{ formatMessage(messages.lookupByEmail) }}
						</template>
						<template #file-lookup>
							<FileIcon aria-hidden="true" /> {{ formatMessage(messages.fileLookup) }}
						</template>
						<template #servers-notices>
							<IssuesIcon aria-hidden="true" /> {{ formatMessage(messages.manageServerNotices) }}
						</template>
						<template #servers-nodes> <ServerIcon aria-hidden="true" /> Server Nodes </template>
					</OverflowMenu>
				</ButtonStyled>
				<ButtonStyled type="transparent">
					<OverflowMenu
						v-if="auth.user"
						class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
						position="bottom"
						direction="left"
						:dropdown-id="`${basePopoutId}-create`"
						:aria-label="formatMessage(messages.createNew)"
						:options="[
							{
								id: 'new-project',
								action: (event) => $refs.modal_creation.show(event),
							},
							{
								id: 'new-collection',
								action: (event) => $refs.modal_collection_creation.show(event),
							},
							{ divider: true },
							{
								id: 'new-organization',
								action: (event) => $refs.modal_organization_creation.show(event),
							},
						]"
					>
						<PlusIcon aria-hidden="true" />
						<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
						<template #new-project>
							<BoxIcon aria-hidden="true" /> {{ formatMessage(messages.newProject) }}
						</template>
						<!-- <template #import-project> <BoxImportIcon /> Import project </template>-->
						<template #new-collection>
							<CollectionIcon aria-hidden="true" /> {{ formatMessage(messages.newCollection) }}
						</template>
						<template #new-organization>
							<OrganizationIcon aria-hidden="true" /> {{ formatMessage(messages.newOrganization) }}
						</template>
					</OverflowMenu>
				</ButtonStyled>
				<OverflowMenu
					v-if="auth.user"
					:dropdown-id="`${basePopoutId}-user`"
					class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
					:options="userMenuOptions"
				>
					<Avatar :src="auth.user.avatar_url" aria-hidden="true" circle />
					<DropdownIcon class="h-5 w-5 text-secondary" />
					<template #profile>
						<UserIcon aria-hidden="true" /> {{ formatMessage(messages.profile) }}
					</template>
					<template #notifications>
						<BellIcon aria-hidden="true" /> {{ formatMessage(commonMessages.notificationsLabel) }}
					</template>
					<template #saved>
						<BookmarkIcon aria-hidden="true" /> {{ formatMessage(messages.savedProjects) }}
					</template>
					<template #servers>
						<ServerIcon aria-hidden="true" /> {{ formatMessage(commonMessages.serversLabel) }}
					</template>
					<template #plus>
						<ArrowBigUpDashIcon aria-hidden="true" />
						{{ formatMessage(messages.upgradeToModrinthPlus) }}
					</template>
					<template #settings>
						<SettingsIcon aria-hidden="true" /> {{ formatMessage(commonMessages.settingsLabel) }}
					</template>
					<template #flags>
						<ReportIcon aria-hidden="true" /> {{ formatMessage(messages.featureFlags) }}
					</template>
					<template #projects>
						<BoxIcon aria-hidden="true" /> {{ formatMessage(messages.projects) }}
					</template>
					<template #organizations>
						<OrganizationIcon aria-hidden="true" /> {{ formatMessage(messages.organizations) }}
					</template>
					<template #revenue>
						<CurrencyIcon aria-hidden="true" /> {{ formatMessage(messages.revenue) }}
					</template>
					<template #analytics>
						<ChartIcon aria-hidden="true" /> {{ formatMessage(messages.analytics) }}
					</template>
					<template #moderation>
						<ScaleIcon aria-hidden="true" /> {{ formatMessage(commonMessages.moderationLabel) }}
					</template>
					<template #sign-out>
						<LogOutIcon aria-hidden="true" /> {{ formatMessage(commonMessages.signOutButton) }}
					</template>
				</OverflowMenu>
				<template v-else>
					<ButtonStyled color="brand">
						<nuxt-link to="/auth/sign-in">
							<LogInIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.signInButton) }}
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled circular>
						<nuxt-link :v-tooltip="formatMessage(commonMessages.settingsLabel)" to="/settings">
							<SettingsIcon :aria-label="formatMessage(commonMessages.settingsLabel)" />
						</nuxt-link>
					</ButtonStyled>
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
					<NuxtLink
						v-for="navRoute in navRoutes"
						:key="navRoute.href"
						:to="navRoute.href"
						class="iconified-button"
					>
						{{ navRoute.label }}
					</NuxtLink>
				</div>
			</div>
			<div
				class="nav-menu nav-menu-mobile"
				:class="{ expanded: isMobileMenuOpen }"
				@focusin="isMobileMenuOpen = true"
				@focusout="isMobileMenuOpen = false"
			>
				<div class="account-container">
					<NuxtLink
						v-if="auth.user"
						:to="`/user/${auth.user.username}`"
						class="iconified-button account-button"
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
					</NuxtLink>
					<nuxt-link v-else class="iconified-button brand-button" to="/auth/sign-in">
						<LogInIcon aria-hidden="true" /> {{ formatMessage(commonMessages.signInButton) }}
					</nuxt-link>
				</div>
				<div class="links">
					<template v-if="auth.user">
						<button class="iconified-button danger-button" @click="logoutUser()">
							<LogOutIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.signOutButton) }}
						</button>
						<button class="iconified-button" @click="$refs.modal_creation.show()">
							<PlusIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.createAProjectButton) }}
						</button>
						<NuxtLink class="iconified-button" to="/dashboard/collections">
							<LibraryIcon class="icon" />
							{{ formatMessage(commonMessages.collectionsLabel) }}
						</NuxtLink>
						<NuxtLink class="iconified-button" to="/servers/manage">
							<ServerIcon class="icon" />
							{{ formatMessage(commonMessages.serversLabel) }}
						</NuxtLink>
						<NuxtLink
							v-if="auth.user.role === 'moderator' || auth.user.role === 'admin'"
							class="iconified-button"
							to="/moderation"
						>
							<ScaleIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.moderationLabel) }}
						</NuxtLink>
						<NuxtLink v-if="flags.developerMode" class="iconified-button" to="/flags">
							<ReportIcon aria-hidden="true" />
							{{ formatMessage(messages.featureFlags) }}
						</NuxtLink>
					</template>
					<NuxtLink class="iconified-button" to="/settings">
						<SettingsIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.settingsLabel) }}
					</NuxtLink>
					<button class="iconified-button" @click="changeTheme">
						<MoonIcon v-if="$theme.active === 'light'" class="icon" />
						<SunIcon v-else class="icon" />
						<span class="dropdown-item__text">
							{{ formatMessage(messages.changeTheme) }}
						</span>
					</button>
				</div>
			</div>
			<div class="mobile-navbar" :class="{ expanded: isBrowseMenuOpen || isMobileMenuOpen }">
				<NuxtLink
					to="/"
					class="tab button-animation"
					:title="formatMessage(navMenuMessages.home)"
					:aria-label="formatMessage(navMenuMessages.home)"
				>
					<HomeIcon aria-hidden="true" />
				</NuxtLink>
				<button
					class="tab button-animation"
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
				</button>
				<template v-if="auth.user">
					<NuxtLink
						to="/dashboard/notifications"
						class="tab button-animation"
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
					</NuxtLink>
					<NuxtLink
						to="/dashboard"
						class="tab button-animation"
						:aria-label="formatMessage(commonMessages.dashboardLabel)"
						:title="formatMessage(commonMessages.dashboardLabel)"
					>
						<ChartIcon aria-hidden="true" />
					</NuxtLink>
				</template>
				<button
					class="tab button-animation"
					:title="formatMessage(messages.toggleMenu)"
					:aria-label="isMobileMenuOpen ? 'Close menu' : 'Open menu'"
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
				</button>
			</div>
		</header>
		<main class="min-h-[calc(100vh-4.5rem-310.59px)]">
			<ProjectCreateModal v-if="auth.user" ref="modal_creation" />
			<CollectionCreateModal ref="modal_collection_creation" />
			<OrganizationCreateModal ref="modal_organization_creation" />
			<slot id="main" />
		</main>
		<footer
			class="footer-brand-background experimental-styles-within border-0 border-t-[1px] border-solid"
		>
			<div class="mx-auto flex max-w-screen-xl flex-col gap-6 p-6 pb-20 sm:px-12 md:py-12">
				<div
					class="grid grid-cols-1 gap-4 text-primary md:grid-cols-[1fr_2fr] lg:grid-cols-[auto_auto_auto_auto_auto]"
				>
					<div
						class="flex flex-col items-center gap-3 md:items-start"
						role="region"
						:aria-label="formatMessage(messages.modrinthInformation)"
					>
						<TextLogo
							aria-hidden="true"
							class="text-logo button-base h-6 w-auto text-contrast lg:h-8"
							@click="developerModeIncrement()"
						/>
						<div class="flex flex-wrap justify-center gap-px sm:-mx-2">
							<ButtonStyled
								v-for="(social, index) in socialLinks"
								:key="`footer-social-${index}`"
								circular
								type="transparent"
							>
								<a
									v-tooltip="social.label"
									:href="social.href"
									target="_blank"
									:rel="`noopener${social.rel ? ` ${social.rel}` : ''}`"
								>
									<component :is="social.icon" class="h-5 w-5" />
								</a>
							</ButtonStyled>
						</div>
						<div class="mt-auto flex flex-wrap justify-center gap-3 md:flex-col">
							<p class="m-0">
								<IntlFormatted :message-id="footerMessages.openSource">
									<template #github-link="{ children }">
										<a
											href="https://github.com/modrinth/code"
											class="text-brand hover:underline"
											target="_blank"
											rel="noopener"
										>
											<component :is="() => children" />
										</a>
									</template>
								</IntlFormatted>
							</p>
							<p class="m-0">© 2025 Rinth, Inc.</p>
						</div>
					</div>
					<div class="mt-4 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:contents">
						<div
							v-for="group in footerLinks"
							:key="group.label"
							class="flex flex-col items-center gap-3 sm:items-start"
						>
							<h3 class="m-0 text-base text-contrast">{{ group.label }}</h3>
							<template v-for="item in group.links" :key="item.label">
								<nuxt-link
									v-if="item.href.startsWith('/')"
									:to="item.href"
									class="w-fit hover:underline"
								>
									{{ item.label }}
								</nuxt-link>
								<a
									v-else
									:href="item.href"
									class="w-fit hover:underline"
									target="_blank"
									rel="noopener"
								>
									{{ item.label }}
								</a>
							</template>
						</div>
					</div>
				</div>
				<div class="flex justify-center text-center text-xs font-medium text-secondary opacity-50">
					{{ formatMessage(footerMessages.legalDisclaimer) }}
				</div>
			</div>
		</footer>
	</div>
</template>
<script setup>
import {
	ArrowBigUpDashIcon,
	BellIcon,
	BlueskyIcon,
	BookmarkIcon,
	BookTextIcon,
	BoxIcon,
	BracesIcon,
	ChartIcon,
	CollectionIcon,
	CompassIcon,
	CurrencyIcon,
	DiscordIcon,
	DownloadIcon,
	DropdownIcon,
	FileIcon,
	FileTextIcon,
	GithubIcon,
	GlassesIcon,
	HamburgerIcon,
	HomeIcon,
	IssuesIcon,
	LibraryIcon,
	LogInIcon,
	LogOutIcon,
	MastodonIcon,
	MessageIcon,
	ModrinthIcon,
	MoonIcon,
	OrganizationIcon,
	PackageOpenIcon,
	PaintbrushIcon,
	PlugIcon,
	PlusIcon,
	ReportIcon,
	ScaleIcon,
	SearchIcon,
	ServerIcon,
	SettingsIcon,
	SunIcon,
	TwitterIcon,
	UserIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ButtonStyled,
	commonMessages,
	commonProjectTypeCategoryMessages,
	injectNotificationManager,
	OverflowMenu,
	PagewideBanner,
} from '@modrinth/ui'
import { isAdmin, isStaff } from '@modrinth/utils'
import { IntlFormatted } from '@vintl/vintl/components'

import TextLogo from '~/components/brand/TextLogo.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import OrganizationCreateModal from '~/components/ui/create/OrganizationCreateModal.vue'
import ProjectCreateModal from '~/components/ui/create/ProjectCreateModal.vue'
import CreatorTaxFormModal from '~/components/ui/dashboard/CreatorTaxFormModal.vue'
import TeleportOverflowMenu from '~/components/ui/servers/TeleportOverflowMenu.vue'
import { errors as generatedStateErrors } from '~/generated/state.json'
import { getProjectTypeMessage } from '~/utils/i18n-project-type.ts'

const country = useUserCountry()

const { formatMessage } = useVIntl()

const auth = await useAuth()
const user = await useUser()

const { addNotification } = injectNotificationManager()

const cosmetics = useCosmetics()
const flags = useFeatureFlags()

const config = useRuntimeConfig()
const route = useNativeRoute()
const router = useNativeRouter()
const link = config.public.siteUrl + route.path.replace(/\/+$/, '')

const { data: payoutBalance } = await useAsyncData('payout/balance', () =>
	useBaseFetch('payout/balance', { apiVersion: 3 }),
)

const showTaxComplianceBanner = computed(() => {
	if (flags.value.testTaxForm && auth.value.user) return true
	const bal = payoutBalance.value
	if (!bal) return false
	const thresholdMet = (bal.withdrawn_ytd ?? 0) >= 600
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

const taxBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.tax.title',
		defaultMessage: 'Tax form required',
	},
	description: {
		id: 'layout.banner.tax.description',
		defaultMessage:
			"You've already withdrawn over $600 from Modrinth this year. To comply with tax regulations, you need to complete a tax form. Your withdrawals are paused until this form is submitted.",
	},
	action: {
		id: 'layout.banner.tax.action',
		defaultMessage: 'Complete tax form',
	},
})

const tinMismatchBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.tin-mismatch.title',
		defaultMessage: 'Tax form failed',
	},
	description: {
		id: 'layout.banner.tin-mismatch.description',
		defaultMessage:
			"Your withdrawals are temporarily locked because your TIN or SSN didn't match IRS records. Please contact support to reset and resubmit your tax form.",
	},
	action: {
		id: 'layout.banner.tin-mismatch.action',
		defaultMessage: 'Contact support',
	},
})

const taxFormModalRef = ref(null)
function openTaxForm(e) {
	if (taxFormModalRef.value && taxFormModalRef.value.startTaxForm) {
		taxFormModalRef.value.startTaxForm(e)
	}
}

const basePopoutId = useId()
async function handleResendEmailVerification() {
	try {
		await resendVerifyEmail()
		addNotification({
			title: 'Verification email sent',
			text: 'Please check your inbox for the verification email.',
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err.data.description,
			type: 'error',
		})
	}
}

const verifyEmailBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.account-action',
		defaultMessage: 'Account action required',
	},
	description: {
		id: 'layout.banner.verify-email.description',
		defaultMessage:
			'For security reasons, Modrinth needs you to verify the email address associated with your account.',
	},
	action: {
		id: 'layout.banner.verify-email.action',
		defaultMessage: 'Re-send verification email',
	},
})

const addEmailBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.account-action',
		defaultMessage: 'Account action required',
	},
	description: {
		id: 'layout.banner.add-email.description',
		defaultMessage:
			'For security reasons, Modrinth needs you to register an email address to your account.',
	},
	action: {
		id: 'layout.banner.add-email.button',
		defaultMessage: 'Visit account settings',
	},
})

const subscriptionPaymentFailedBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.subscription-payment-failed.title',
		defaultMessage: 'Billing action required.',
	},
	description: {
		id: 'layout.banner.subscription-payment-failed.description',
		defaultMessage:
			'One or more subscriptions failed to renew. Please update your payment method to prevent losing access!',
	},
	action: {
		id: 'layout.banner.subscription-payment-failed.button',
		defaultMessage: 'Update billing info',
	},
})

const stagingBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.staging.title',
		defaultMessage: 'You’re viewing Modrinth’s staging environment',
	},
	description: {
		id: 'layout.banner.staging.description',
		defaultMessage:
			'The staging environment is completely separate from the production Modrinth database. This is used for testing and debugging purposes, and may be running in-development versions of the Modrinth backend or frontend newer than the production instance.',
	},
})

const failedToBuildBannerMessages = defineMessages({
	title: {
		id: 'layout.banner.build-fail.title',
		defaultMessage: 'Error generating state from API when building.',
	},
	description: {
		id: 'layout.banner.build-fail.description',
		defaultMessage:
			"This deploy of Modrinth's frontend failed to generate state from the API. This may be due to an outage or an error in configuration. Rebuild when the API is available. Error codes: {errors}; Current API URL is: {url}",
	},
})

const navMenuMessages = defineMessages({
	home: {
		id: 'layout.nav.home',
		defaultMessage: 'Home',
	},
	search: {
		id: 'layout.nav.search',
		defaultMessage: 'Search',
	},
	discoverContent: {
		id: 'layout.nav.discover-content',
		defaultMessage: 'Discover content',
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
	modrinthInformation: {
		id: 'layout.footer.modrinth-information',
		defaultMessage: 'Modrinth information',
	},
	createNew: {
		id: 'layout.action.create-new',
		defaultMessage: 'Create new...',
	},
	reviewProjects: {
		id: 'layout.action.review-projects',
		defaultMessage: 'Review projects',
	},
	reports: {
		id: 'layout.action.reports',
		defaultMessage: 'Reports',
	},
	lookupByEmail: {
		id: 'layout.action.lookup-by-email',
		defaultMessage: 'Lookup by email',
	},
	fileLookup: {
		id: 'layout.action.file-lookup',
		defaultMessage: 'File lookup',
	},
	manageServerNotices: {
		id: 'layout.action.manage-server-notices',
		defaultMessage: 'Manage server notices',
	},
	newProject: {
		id: 'layout.action.new-project',
		defaultMessage: 'New project',
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
	featureFlags: {
		id: 'layout.nav.feature-flags',
		defaultMessage: 'Feature flags',
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
	analytics: {
		id: 'layout.nav.analytics',
		defaultMessage: 'Analytics',
	},
})

const footerMessages = defineMessages({
	openSource: {
		id: 'layout.footer.open-source',
		defaultMessage: 'Modrinth is <github-link>open source</github-link>.',
	},
	legalDisclaimer: {
		id: 'layout.footer.legal-disclaimer',
		defaultMessage:
			'NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.',
	},
})

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

const developerModeCounter = ref(0)

const isMobileMenuOpen = ref(false)
const isBrowseMenuOpen = ref(false)
const navRoutes = computed(() => [
	{
		id: 'mods',
		label: formatMessage(getProjectTypeMessage('mod', true)),
		href: '/mods',
	},
	{
		label: formatMessage(getProjectTypeMessage('plugin', true)),
		href: '/plugins',
	},
	{
		label: formatMessage(getProjectTypeMessage('datapack', true)),
		href: '/datapacks',
	},
	{
		label: formatMessage(getProjectTypeMessage('shader', true)),
		href: '/shaders',
	},
	{
		label: formatMessage(getProjectTypeMessage('resourcepack', true)),
		href: '/resourcepacks',
	},
	{
		label: formatMessage(getProjectTypeMessage('modpack', true)),
		href: '/modpacks',
	},
])

const userMenuOptions = computed(() => {
	let options = [
		{
			id: 'profile',
			link: `/user/${auth.value.user.username}`,
		},
		{
			id: 'plus',
			link: '/plus',
			color: 'purple',
			shown: !flags.value.hidePlusPromoInUserMenu && !isPermission(auth.value.user.badges, 1 << 0),
		},
		{
			id: 'notifications',
			link: '/dashboard/notifications',
		},
		{
			id: 'saved',
			link: '/dashboard/collections',
		},
		{
			id: 'servers',
			link: '/servers/manage',
		},
		{
			id: 'flags',
			link: '/flags',
			shown: flags.value.developerMode,
		},
		{
			id: 'settings',
			link: '/settings',
		},
	]

	// TODO: Only show if user has projects
	options = [
		...options,
		{
			divider: true,
		},
		{
			id: 'projects',
			link: '/dashboard/projects',
		},
		{
			id: 'organizations',
			link: '/dashboard/organizations',
		},
		{
			id: 'revenue',
			link: '/dashboard/revenue',
		},
		{
			id: 'analytics',
			link: '/dashboard/analytics',
		},
	]

	options = [
		...options,
		{
			divider: true,
		},
		{
			id: 'sign-out',
			color: 'danger',
			action: () => logoutUser(),
			hoverFilled: true,
		},
	]
	return options
})

const isDiscovering = computed(
	() => route.name && route.name.startsWith('search-') && !route.query.sid,
)

const isDiscoveringSubpage = computed(
	() => route.name && route.name.startsWith('type-id') && !route.query.sid,
)

const isRussia = computed(() => country.value === 'ru')

const rCount = ref(0)

const randomProjects = ref([])
const disableRandomProjects = ref(false)

const disableRandomProjectsForRoute = computed(
	() =>
		route.name.startsWith('servers') ||
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
			randomProjects.value = await useBaseFetch('projects_random?count=50').catch((err) => {
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

function developerModeIncrement() {
	if (developerModeCounter.value >= 5) {
		flags.value.developerMode = !flags.value.developerMode
		developerModeCounter.value = 0
		saveFeatureFlags()
		if (flags.value.developerMode) {
			addNotification({
				title: 'Developer mode activated',
				text: 'Developer mode has been enabled',
				type: 'success',
			})
		} else {
			addNotification({
				title: 'Developer mode deactivated',
				text: 'Developer mode has been disabled',
				type: 'success',
			})
		}
	} else {
		developerModeCounter.value++
	}
}

async function logoutUser() {
	await logout()
}

function runAnalytics() {
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

const { cycle: changeTheme } = useTheme()

function hideStagingBanner() {
	cosmetics.value.hideStagingBanner = true
}

function hideRussiaCensorshipBanner() {
	flags.value.hideRussiaCensorshipBanner = true
	saveFeatureFlags()
}

const socialLinks = [
	{
		label: formatMessage(
			defineMessage({ id: 'layout.footer.social.discord', defaultMessage: 'Discord' }),
		),
		href: 'https://discord.modrinth.com',
		icon: DiscordIcon,
	},
	{
		label: formatMessage(
			defineMessage({ id: 'layout.footer.social.bluesky', defaultMessage: 'Bluesky' }),
		),
		href: 'https://bsky.app/profile/modrinth.com',
		icon: BlueskyIcon,
	},
	{
		label: formatMessage(
			defineMessage({ id: 'layout.footer.social.mastodon', defaultMessage: 'Mastodon' }),
		),
		href: 'https://floss.social/@modrinth',
		icon: MastodonIcon,
		rel: 'me',
	},
	{
		label: formatMessage(defineMessage({ id: 'layout.footer.social.x', defaultMessage: 'X' })),
		href: 'https://x.com/modrinth',
		icon: TwitterIcon,
	},
	{
		label: formatMessage(
			defineMessage({ id: 'layout.footer.social.github', defaultMessage: 'GitHub' }),
		),
		href: 'https://github.com/modrinth',
		icon: GithubIcon,
	},
]

const footerLinks = [
	{
		label: formatMessage(defineMessage({ id: 'layout.footer.about', defaultMessage: 'About' })),
		links: [
			{
				href: '/news',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.about.news', defaultMessage: 'News' }),
				),
			},
			{
				href: '/news/changelog',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.about.changelog', defaultMessage: 'Changelog' }),
				),
			},
			{
				href: 'https://status.modrinth.com',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.about.status', defaultMessage: 'Status' }),
				),
			},
			{
				href: 'https://careers.modrinth.com',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.about.careers', defaultMessage: 'Careers' }),
				),
			},
			{
				href: '/legal/cmp-info',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.about.rewards-program',
						defaultMessage: 'Rewards Program',
					}),
				),
			},
		],
	},
	{
		label: formatMessage(
			defineMessage({ id: 'layout.footer.products', defaultMessage: 'Products' }),
		),
		links: [
			{
				href: '/plus',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.products.plus', defaultMessage: 'Modrinth+' }),
				),
			},
			{
				href: '/app',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.products.app', defaultMessage: 'Modrinth App' }),
				),
			},
			{
				href: '/servers',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.products.servers',
						defaultMessage: 'Modrinth Servers',
					}),
				),
			},
		],
	},
	{
		label: formatMessage(
			defineMessage({ id: 'layout.footer.resources', defaultMessage: 'Resources' }),
		),
		links: [
			{
				href: 'https://support.modrinth.com',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.resources.help-center',
						defaultMessage: 'Help Center',
					}),
				),
			},
			{
				href: 'https://translate.modrinth.com',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.resources.translate', defaultMessage: 'Translate' }),
				),
			},
			{
				href: 'https://github.com/modrinth/code/issues',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.resources.report-issues',
						defaultMessage: 'Report issues',
					}),
				),
			},
			{
				href: 'https://docs.modrinth.com/api/',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.resources.api-docs',
						defaultMessage: 'API documentation',
					}),
				),
			},
		],
	},
	{
		label: formatMessage(defineMessage({ id: 'layout.footer.legal', defaultMessage: 'Legal' })),
		links: [
			{
				href: '/legal/rules',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.legal.rules', defaultMessage: 'Content Rules' }),
				),
			},
			{
				href: '/legal/terms',
				label: formatMessage(
					defineMessage({ id: 'layout.footer.legal.terms-of-use', defaultMessage: 'Terms of Use' }),
				),
			},
			{
				href: '/legal/privacy',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.legal.privacy-policy',
						defaultMessage: 'Privacy Policy',
					}),
				),
			},
			{
				href: '/legal/security',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.legal.security-notice',
						defaultMessage: 'Security Notice',
					}),
				),
			},
			{
				href: '/legal/copyright',
				label: formatMessage(
					defineMessage({
						id: 'layout.footer.legal.copyright-policy',
						defaultMessage: 'Copyright Policy and DMCA',
					}),
				),
			},
		],
	},
]
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
		z-index: 6;
		transform: translateY(100%);
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

			.iconified-button {
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
				padding-bottom: 0;

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
		z-index: 7;
		width: 100%;
		align-items: center;
		justify-content: space-between;
		transition: border-radius 0.3s ease-out;
		border-top: 2px solid rgba(0, 0, 0, 0);
		box-sizing: border-box;

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

@media (any-hover: none) and (max-width: 640px) {
	.desktop-only {
		display: none;
	}
}

@media (any-hover: none) and (max-width: 640px) {
	.mobile-navigation {
		display: flex;
	}
}

.footer-brand-background {
	background: var(--brand-gradient-strong-bg);
	border-color: var(--brand-gradient-border);
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
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
