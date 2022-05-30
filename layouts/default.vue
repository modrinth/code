<template>
  <div class="layout">
    <header class="site-header" role="presentation">
      <section class="navbar columns" role="navigation">
        <section class="skip column" role="presentation">
          <a href="#main">Skip to Main Content</a>
          <a v-if="registeredSkipLink" :href="registeredSkipLink.id">{{
            registeredSkipLink.text
          }}</a>
        </section>
        <section class="logo column" role="presentation">
          <NuxtLink to="/" aria-label="Modrinth home page">
            <ModrinthLogo aria-hidden="true" class="text-logo" />
          </NuxtLink>
        </section>
        <section class="nav-group columns" role="presentation">
          <section class="nav" aria-label="Page links">
            <div class="styled-tabs">
              <NuxtLink to="/mods" class="tab">
                <span>Mods</span>
              </NuxtLink>
              <NuxtLink to="/modpacks" class="tab tab--alpha">
                <span>Modpacks</span>
              </NuxtLink>
            </div>
          </section>
          <section class="column-grow user-outer" aria-label="Account links">
            <section class="user-controls">
              <button
                class="control-button"
                title="Switch theme"
                @click="changeTheme"
              >
                <MoonIcon
                  v-if="$colorMode.value === 'light'"
                  aria-hidden="true"
                />
                <SunIcon v-else aria-hidden="true" />
              </button>
              <nuxt-link
                v-if="$auth.user"
                to="/create/project"
                class="control-button"
                title="Create project"
              >
                <PlusIcon aria-hidden="true" />
              </nuxt-link>
              <nuxt-link
                v-if="$auth.user"
                to="/notifications"
                class="control-button"
                title="Notifications"
              >
                <NotificationIcon aria-hidden="true" />
                <div v-if="$user.notifications.length > 0" class="bubble">
                  {{ $user.notifications.length }}
                </div>
              </nuxt-link>
              <div v-if="$auth.user" ref="mobileMenu" class="dropdown">
                <button class="control" value="Profile Dropdown">
                  <img
                    :src="$auth.user.avatar_url"
                    class="user-icon"
                    aria-hidden="true"
                    alt="Your avatar"
                  />
                  <DropdownIcon class="caret" />
                </button>
                <ul class="content card" @click="removeFocus">
                  <li>
                    <NuxtLink
                      class="item"
                      :to="`/user/${$auth.user.username}`"
                      @click="removeFocus"
                    >
                      <div class="title profile-link">
                        <div class="username">@{{ $auth.user.username }}</div>
                        <div class="prompt">Go to my profile</div>
                      </div>
                    </NuxtLink>
                  </li>
                  <hr class="divider" />
                  <li>
                    <NuxtLink class="item" to="/create/project">
                      <PlusIcon class="icon" />
                      <span class="title">Create a project</span>
                    </NuxtLink>
                  </li>
                  <hr class="divider" />
                  <li>
                    <NuxtLink class="item" to="/notifications">
                      <NotificationIcon class="icon" />
                      <span class="title">Notifications</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink class="item" to="/settings/follows">
                      <HeartIcon class="icon" />
                      <span class="title">Following</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink class="item" to="/settings">
                      <SettingsIcon class="icon" />
                      <span class="title">Settings</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink
                      v-if="
                        $auth.user.role === 'moderator' ||
                        $auth.user.role === 'admin'
                      "
                      class="item"
                      to="/moderation"
                    >
                      <ModerationIcon class="icon" />
                      <span class="title">Moderation</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <button class="item" @click="changeTheme">
                      <MoonIcon
                        v-if="$colorMode.value === 'light'"
                        class="icon"
                      />
                      <SunIcon v-else class="icon" />
                      <span class="dropdown-item__text">Change theme</span>
                    </button>
                  </li>
                  <hr class="divider" />
                  <li>
                    <button class="item" @click="logout">
                      <LogOutIcon class="icon" />
                      <span class="dropdown-item__text">Log out</span>
                    </button>
                  </li>
                </ul>
              </div>
              <section v-else class="auth-prompt">
                <a :href="authUrl" class="log-in-button">
                  <GitHubIcon aria-hidden="true" />
                  Sign in with GitHub</a
                >
              </section>
            </section>
          </section>
        </section>
      </section>
      <section class="mobile-navbar">
        <NuxtLink to="/" class="tab">
          <HomeIcon />
          <span>Home</span>
        </NuxtLink>
        <NuxtLink to="/mods" class="tab">
          <ModIcon />
          <span>Mods</span>
        </NuxtLink>
        <NuxtLink to="/modpacks" class="tab">
          <ModpackIcon />
          <span>Modpacks</span>
        </NuxtLink>
        <button class="tab" @click="toggleMobileMenu()">
          <HamburgerIcon />
          <span>{{ isMobileMenuOpen ? 'Less' : 'More' }}</span>
        </button>
      </section>
      <section ref="mobileMenu" class="mobile-menu">
        <div class="mobile-menu-wrapper">
          <div class="items-container rows">
            <NuxtLink
              v-if="$auth.user"
              class="item user-item"
              :to="`/user/${$auth.user.username}`"
            >
              <img :src="$auth.user.avatar_url" class="user-icon" />
              <div class="profile-link">
                <div class="username">@{{ $auth.user.username }}</div>
                <div class="prompt">Go to my profile</div>
              </div>
            </NuxtLink>
            <button v-if="$auth.user" class="item log-out" @click="logout">
              <LogOutIcon class="icon" />
              <span class="dropdown-item__text">Log out</span>
            </button>
            <NuxtLink v-if="$auth.user" class="item" to="/create/project">
              <PlusIcon class="icon" />
              <span class="title">Create a project</span>
            </NuxtLink>
            <NuxtLink v-if="$auth.user" class="item" to="/settings">
              <SettingsIcon class="icon" />
              <span class="title">Settings</span>
            </NuxtLink>
            <NuxtLink
              v-if="
                $auth.user &&
                ($auth.user.role === 'moderator' || $auth.user.role === 'admin')
              "
              class="item"
              to="/moderation"
            >
              <ModerationIcon class="icon" />
              <span class="title">Moderation</span>
            </NuxtLink>
            <NuxtLink v-if="$auth.user" class="item" to="/settings/follows">
              <HeartIcon class="icon" />
              <span class="title">Following</span>
            </NuxtLink>
            <NuxtLink v-if="$auth.user" class="item" to="/notifications">
              <NotificationIcon class="icon" />
              <span class="title">Notifications</span>
            </NuxtLink>
            <button class="item" @click="changeTheme">
              <MoonIcon v-if="$colorMode.value === 'light'" class="icon" />
              <SunIcon v-else class="icon" />
              <span class="dropdown-item__text">Change theme</span>
            </button>
            <a v-if="!$auth.user" :href="authUrl" class="item log-in">
              <GitHubIcon aria-hidden="true" />
              Sign in with GitHub</a
            >
          </div>
        </div>
      </section>
    </header>
    <main>
      <CookieConsent />
      <notifications
        group="main"
        position="bottom right"
        :max="5"
        :ignore-duplicates="true"
      />
      <Nuxt id="main" />
    </main>
    <footer>
      <div class="logo-info" role="region" aria-label="Modrinth information">
        <ModrinthLogo aria-hidden="true" class="text-logo" />
        <p>
          Modrinth is open source software. You may view the source code at
          <a
            target="_blank"
            href="https://github.com/modrinth/knossos"
            class="text-link"
          >
            our GitHub page</a
          >
        </p>
        <p>{{ owner }}/{{ slug }} {{ branch }}@{{ hash.substring(0, 7) }}</p>
        <p>© Rinth, Inc.</p>
      </div>
      <div class="links links-1" role="region" aria-label="Legal">
        <h4 aria-hidden="true">Legal</h4>
        <nuxt-link to="/legal/terms">Terms</nuxt-link>
        <nuxt-link to="/legal/privacy">Privacy</nuxt-link>
        <nuxt-link to="/legal/rules">Rules</nuxt-link>
        <a
          target="_blank"
          href="https://github.com/modrinth/knossos/blob/master/LICENSE.md"
        >
          License
        </a>
      </div>
      <div class="links links-2" role="region" aria-label="Resources">
        <h4 aria-hidden="true">Resources</h4>
        <a target="_blank" href="https://blog.modrinth.com">Blog</a>
        <a target="_blank" href="https://discord.gg/EUHuJHt">Discord</a>
        <a target="_blank" href="https://github.com/modrinth/knossos">GitHub</a>
        <a target="_blank" href="https://docs.modrinth.com">Docs</a>
      </div>
      <div class="buttons">
        <nuxt-link to="/settings/privacy" class="iconified-button">
          <ShieldIcon aria-hidden="true" />
          Privacy settings
        </nuxt-link>
        <button class="iconified-button" @click="changeTheme">
          <MoonIcon v-if="$colorMode.value === 'light'" aria-hidden="true" />
          <SunIcon v-else aria-hidden="true" />
          Change theme
        </button>
      </div>
    </footer>
  </div>
</template>

<script>
import ClickOutside from 'vue-click-outside'

import ModrinthLogo from '~/assets/images/text-logo.svg?inline'

import HamburgerIcon from '~/assets/images/utils/hamburger.svg?inline'

import NotificationIcon from '~/assets/images/sidebar/notifications.svg?inline'
import SettingsIcon from '~/assets/images/sidebar/settings.svg?inline'
import ShieldIcon from '~/assets/images/utils/shield.svg?inline'
import ModerationIcon from '~/assets/images/sidebar/admin.svg?inline'
import HomeIcon from '~/assets/images/sidebar/home.svg?inline'
import ModIcon from '~/assets/images/sidebar/mod.svg?inline'
import ModpackIcon from '~/assets/images/sidebar/modpack.svg?inline'
import MoonIcon from '~/assets/images/utils/moon.svg?inline'

import SunIcon from '~/assets/images/utils/sun.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import DropdownIcon from '~/assets/images/utils/dropdown.svg?inline'
import LogOutIcon from '~/assets/images/utils/log-out.svg?inline'
import HeartIcon from '~/assets/images/utils/heart.svg?inline'

import GitHubIcon from '~/assets/images/utils/github.svg?inline'

import CookieConsent from '~/components/ads/CookieConsent'

const overflowStyle = 'scroll'

export default {
  components: {
    ModrinthLogo,
    MoonIcon,
    SunIcon,
    LogOutIcon,
    GitHubIcon,
    NotificationIcon,
    HomeIcon,
    ModIcon,
    ModpackIcon,
    HamburgerIcon,
    CookieConsent,
    SettingsIcon,
    ShieldIcon,
    ModerationIcon,
    PlusIcon,
    DropdownIcon,
    HeartIcon,
  },
  directives: {
    ClickOutside,
  },
  data() {
    return {
      isDropdownOpen: false,
      owner: process.env.owner || 'modrinth',
      slug: process.env.slug || 'knossos',
      branch: process.env.branch || 'master',
      hash: process.env.hash || 'unknown',
      isMobileMenuOpen: false,
      registeredSkipLink: null,
    }
  },
  async fetch() {
    await Promise.all([
      this.$store.dispatch('user/fetchAll', { force: true }),
      this.$store.dispatch('tag/fetchAllTags'),
      this.$store.dispatch('cosmetics/fetchCosmetics', this.$cookies),
    ])
  },
  computed: {
    authUrl() {
      return `${this.$axios.defaults.baseURL}auth/init?url=${process.env.domain}${this.$route.path}`
    },
  },
  watch: {
    $route() {
      this.$refs.mobileMenu.className = 'mobile-menu'
      this.isMobileMenuOpen =
        this.$refs.mobileMenu.className === 'mobile-menu active'

      document.body.style.overflowY = overflowStyle

      this.$store.dispatch('user/fetchAll')
    },
  },
  beforeCreate() {
    if (this.$route.query.code) {
      this.$router.push(this.$route.path)
    }
  },
  created() {
    this.$nuxt.$on('registerSkipLink', (data) => {
      this.registeredSkipLink = data
    })
  },
  methods: {
    toggleMobileMenu() {
      window.scrollTo(0, 0)
      const currentlyActive =
        this.$refs.mobileMenu.className === 'mobile-menu active'
      this.$refs.mobileMenu.className = `mobile-menu${
        currentlyActive ? '' : ' active'
      }`
      document.body.scrollTop = 0

      document.body.style.overflowY =
        document.body.style.overflowY !== 'hidden' ? 'hidden' : overflowStyle

      this.isMobileMenuOpen = !currentlyActive
    },
    async logout() {
      this.$cookies.set('auth-token-reset', true)
      // If users logs out on dashboard, force redirect on the home page to clear cookies
      if (this.$route.path.startsWith('/settings')) {
        window.location.href = '/'
      } else {
        await this.$router.go(null)

        this.$notify({
          group: 'main',
          title: 'Logged Out',
          text: 'You have logged out successfully!',
          type: 'success',
        })
      }
    },
    changeTheme() {
      this.$colorMode.preference =
        this.$colorMode.value === 'dark' ? 'light' : 'dark'
    },
    removeFocus() {
      document.activeElement.blur() // This doesn't work, sadly. Help
    },
  },
}
</script>

<style lang="scss">
.skip a {
  clip: rect(1px, 1px, 1px, 1px);
  height: 1px;
  overflow: hidden;
  position: absolute;
  white-space: nowrap;
  width: 1px;
}

.skip a:focus {
  clip: auto;
  height: auto;
  overflow: auto;
  position: absolute;
  width: auto;
  padding: 0.5rem 0.75rem;
  background-color: var(--color-brand);
  color: var(--color-brand-inverted);
  border-radius: var(--size-rounded-max);
  margin: 0 0.5rem 0 0;
  box-shadow: inset 0px -1px 1px rgba(17, 24, 39, 0.1);
  z-index: 1;
}

.layout {
  min-height: 100vh;
  background-color: var(--color-bg);
  display: block;

  @media screen and (min-width: 1024px) {
    min-height: calc(100vh - var(--spacing-card-bg));
  }

  @media screen and (max-width: 750px) {
    margin-bottom: calc(var(--size-mobile-navbar-height) + 2rem);
  }

  .site-header {
    margin-top: var(--spacing-card-md);
    margin-bottom: var(--spacing-card-md);
    max-width: 100vw;

    @media screen and (min-width: 1024px) {
      border-radius: var(--size-rounded-sm);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;
    }

    .navbar {
      margin: 0 var(--spacing-card-lg);
      padding: 0 var(--spacing-card-lg);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;

      @media screen and (max-width: 750px) {
        justify-content: center;
      }

      section.logo {
        display: flex;
        justify-content: space-between;
        color: var(--color-text-dark);

        a {
          align-items: center;
          display: flex;
        }

        .small-logo {
          display: block;
        }

        svg {
          height: 1.75rem;
          width: auto;
        }

        .badge {
          margin-left: 0.25rem;
          display: none;
          @media screen and (min-width: 430px) {
            display: unset;
          }
        }

        button {
          background: none;
          border: none;
          margin: 0 0 0 0.5rem;
          padding: 0;

          svg {
            height: 1.5rem;
            width: 1.5rem;
          }
        }
      }

      section.nav-group {
        display: flex;
        flex-grow: 5;

        section.nav {
          flex-grow: 5;

          .styled-tabs {
            display: flex;
            position: relative;
            top: 50%;
            transform: translateY(-50%);
            margin-top: 3px;
            margin-left: 2rem;

            a {
              margin-left: 0;
              margin-right: auto;
            }

            a.tab {
              padding: 0;
              margin-right: 1rem;
              display: flex;
              align-items: flex-start;

              &--alpha::after {
                content: 'Alpha';
                background-color: var(--color-warning-bg);
                color: var(--color-warning-text);
                border-radius: 1rem;
                padding: 0.25rem 0.5rem;
                margin-left: 0.4rem;
                font-size: 0.7rem;
              }
            }
          }
        }

        .user-outer {
          z-index: 20;
        }

        section.user-controls {
          align-items: center;
          display: flex;
          flex-direction: row;
          justify-content: space-between;
          position: relative;
          top: 50%;
          transform: translateY(-50%);
          min-width: 12rem;

          .control-button {
            display: flex;
            max-width: 2rem;
            padding: 0.5rem;
            background-color: var(--color-raised-bg);
            border-radius: var(--size-rounded-max);
            margin: 0 0.5rem 0 0;
            box-shadow: inset 0px -1px 1px rgba(17, 24, 39, 0.1);

            &:hover,
            &:focus-visible {
              background-color: var(--color-button-bg-hover);
              color: var(--color-button-text-hover);
            }

            &:active {
              background-color: var(--color-button-bg-active);
              color: var(--color-button-text-active);
            }

            svg {
              height: 1rem;
              width: 1rem;
            }

            .bubble {
              position: absolute;
              margin-left: 0.5rem;
              bottom: 1rem;
              border-radius: 0.9rem;
              height: 0.8rem;
              padding: 0 0.25rem;
              display: flex;
              justify-content: center;
              align-items: center;

              font-size: 0.6rem;
              background-color: var(--color-brand);
              color: var(--color-brand-inverted);
            }
          }

          .hide-desktop {
            display: none;
          }

          .dropdown {
            position: relative;

            .control {
              align-items: center;
              background: none;
              display: flex;
              justify-content: center;
              padding: 0;

              .user-icon {
                border-radius: 100%;
                height: 2rem;
                outline: 2px solid var(--color-raised-bg);
                width: 2rem;
              }

              .caret {
                color: inherit;
                margin-left: 0.25rem;
                width: 1rem;
              }
            }

            .content {
              border: 1px solid var(--color-divider-dark);
              list-style: none;
              margin: 0.5rem 0 0 0;
              max-width: 25rem;
              min-width: 12rem;
              opacity: 0;
              padding: 1rem;
              position: absolute;
              right: -1rem;
              transform: scaleY(0.9);
              transform-origin: top;
              transition: all 0.1s ease-in-out 0.05s, color 0s ease-in-out 0s,
                background-color 0s ease-in-out 0s,
                border-color 0s ease-in-out 0s;
              visibility: hidden;
              width: max-content;
              z-index: 1;

              .divider {
                background-color: var(--color-divider-dark);
                border: none;
                color: var(--color-divider-dark);
                height: 1px;
                margin: 0.5rem 0;
              }

              .item {
                align-items: center;
                background: none;
                border-radius: 0.5rem;
                box-sizing: border-box;
                color: inherit;
                display: flex;
                padding: 0.5rem;
                width: 100%;

                .icon {
                  margin-right: 0.5rem;
                  height: 20px;
                  width: 20px;
                }
              }

              .item:hover,
              .item:focus {
                background-color: var(--color-bg);
              }

              .profile-link {
                .prompt {
                  margin-top: 0.25rem;
                  color: var(--color-text-secondary);
                }
              }
            }

            @media screen and (max-width: 1300px) {
              .content {
                margin-right: 1rem;
              }
            }
          }

          .dropdown:hover .user-icon,
          .dropdown:focus .user-icon,
          .dropdown:focus-within .user-icon {
            outline-color: var(--color-raised-bg-hover);
          }

          .dropdown:hover .content {
            opacity: 1;
            transform: scaleY(1);
            visibility: visible;
          }
        }

        section.auth-prompt {
          display: flex;
          align-items: center;
          height: 100%;
          margin: 0;

          .log-in-button {
            margin: 0 auto;

            display: flex;
            align-items: center;

            text-align: center;
            border-radius: var(--size-rounded-max);
            background-color: var(--color-brand);
            white-space: nowrap;
            //outline: none; Bad for accessibility
            color: var(--color-brand-inverted);
            padding: 0.5rem 0.75rem;

            svg {
              vertical-align: middle;
              margin-right: 0.5rem;
            }

            &:hover,
            &:focus {
              background-color: var(--color-brand-hover);
            }

            &:active {
              background-color: var(--color-brand-active);
            }
          }
        }
      }

      @media screen and (max-width: 750px) {
        section.nav-group {
          display: none;

          .hide-desktop {
            display: unset;
          }
        }
      }
    }

    .mobile-navbar {
      display: none;
      width: 100%;
      height: var(--size-mobile-navbar-height);
      position: fixed;
      left: 0;
      bottom: 0;
      justify-content: center;
      align-items: center;
      background-color: var(--color-raised-bg);
      box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
      z-index: 6;

      .tab {
        background: none;
        display: flex;
        flex-grow: 1;
        flex-basis: 0;
        justify-content: center;
        align-items: center;
        flex-direction: column;
        font-weight: bold;
        padding: 0;
        margin: auto;
        transition: color ease-in-out 0.15s;
        color: var(--color-text-inactive);

        svg {
          height: 1.75rem;
          width: 1.75rem;
          margin-bottom: 0.25rem;
        }

        &:hover,
        &:focus {
          color: var(--color-text);
        }

        &.nuxt-link-exact-active {
          svg {
            color: var(--color-brand);
          }

          color: var(--color-text);
        }
      }

      @media screen and (max-width: 750px) {
        display: flex;
      }
    }
  }

  .mobile-menu {
    display: none;
    position: absolute;
    top: 0;
    background-color: var(--color-bg);
    height: calc(100% - var(--size-mobile-navbar-height));
    width: 100%;
    z-index: 5;

    .mobile-menu-wrapper {
      max-height: calc(100vh - var(--size-mobile-navbar-height));
      overflow-y: auto;
      margin-top: auto;

      .items-container {
        margin: 1rem 2rem;

        button {
          box-sizing: unset;
        }

        .item {
          padding: 1rem 2rem;
          background-color: var(--color-raised-bg);
          border-radius: var(--size-rounded-md);
          align-items: center;
          justify-content: center;
          display: flex;
          column-gap: 0.25rem;
          width: calc(100% - 4rem);
          max-width: 18rem;

          &.nuxt-link-exact-active {
            color: var(--color-button-text-active);
            svg {
              color: var(--color-brand);
            }
          }

          &.log-in {
            color: var(--color-brand-inverted);
            background-color: var(--color-brand);
          }

          &.log-out {
            color: white;
            background-color: var(--color-badge-red-bg);
          }

          &.user-item {
            flex-direction: column;
            row-gap: 0.5rem;
            //width: 8rem;
            max-width: 18rem;
            flex-grow: 0;

            .profile-link {
              text-align: center;

              .prompt {
                color: var(--color-text-secondary);
              }
            }

            .user-icon {
              width: 4rem;
              height: 4rem;
              border-radius: var(--size-rounded-max);
            }
          }
        }
      }
    }

    div {
      flex-grow: 1;
      justify-content: end;
      align-items: center;
      row-gap: 1rem;
    }

    &.active {
      display: flex;

      @media screen and (min-width: 750px) {
        display: none;
      }
    }
  }

  main {
    grid-area: main;
  }

  footer {
    margin: 6rem 0 2rem 0;
    text-align: center;
    display: grid;
    grid-template:
      'logo-info logo-info' auto
      'links-1   links-2' auto
      'buttons buttons' auto
      / 1fr 1fr;

    .logo-info {
      margin-left: auto;
      margin-right: auto;
      max-width: 20rem;
      margin-bottom: 1rem;
      grid-area: logo-info;

      .text-logo {
        width: 10rem;
        height: auto;
      }
    }

    .links {
      display: flex;
      flex-direction: column;
      margin-bottom: 1rem;

      h4 {
        color: var(--color-text-dark);
        margin: 0 0 1rem 0;
      }

      a {
        margin: 0 0 1rem 0;
      }

      &.links-1 {
        grid-area: links-1;
      }

      &.links-2 {
        grid-area: links-2;
      }
    }

    .buttons {
      margin-left: auto;
      margin-right: auto;
      grid-area: buttons;

      button,
      a {
        background-color: var(--color-raised-bg);

        margin-bottom: 0.5rem;
        margin-left: auto;
        margin-right: auto;

        &:hover,
        &:focus-visible {
          background-color: var(--color-button-bg-hover);
        }
      }
    }

    @media screen and (min-width: 1024px) {
      display: flex;
      text-align: unset;

      .logo-info {
        margin-right: 2rem;
      }

      .links {
        margin-right: 2rem;
      }

      .buttons {
        width: unset;
        margin-left: 0;

        button,
        a {
          margin-right: unset;
        }
      }
    }
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
</style>
<style src="vue-multiselect/dist/vue-multiselect.min.css"></style>
