<template>
  <div class="layout">
    <header class="site-header">
      <section class="navbar columns">
        <section class="logo column">
          <NuxtLink to="/">
            <ModrinthLogo
              v-if="$colorMode.value === 'light'"
              aria-label="modrinth"
            />
            <ModrinthLogoWhite v-else aria-label="modrinth" />
          </NuxtLink>
          <span class="badge yellow">Beta</span>
        </section>
        <section class="column-grow-5 nav">
          <div class="tabs">
            <NuxtLink to="/mods" class="tab">
              <span>Mods</span>
            </NuxtLink>
            <client-only>
              <div v-if="this.$auth.loggedIn" class="section">
                <NuxtLink to="/dashboard/projects" class="tab">
                  <span>Dashboard</span>
                </NuxtLink>
              </div>
            </client-only>
          </div>
        </section>
        <section class="column-grow">
          <client-only>
            <template v-if="this.$auth.loggedIn">
              <section class="user-controls">
                <div
                  v-click-outside="hideDropdown"
                  class="dropdown"
                  :class="{ open: isDropdownOpen }"
                >
                  <button class="control" @click="toggleDropdown">
                    <div class="avatar">
                      <span>{{ this.$auth.user.username }}</span>
                      <img :src="this.$auth.user.avatar_url" class="icon" />
                    </div>
                    <DropdownIcon class="dropdown-icon" />
                  </button>
                  <div class="content">
                    <ul v-if="isDropdownOpen" @click="hideDropdown">
                      <li>
                        <NuxtLink :to="userUrl">
                          <UserIcon />
                          <span>Profile</span>
                        </NuxtLink>
                      </li>
                      <li>
                        <NuxtLink to="/dashboard/notifications">
                          <NotificationIcon />
                          <span>Notifications</span>
                        </NuxtLink>
                      </li>
                      <!--<li v-tooltip="'Not implemented yet'" class="hidden">
                        <NuxtLink :to="userTeamsUrl" disabled>
                          <UsersIcon />
                          <span>Teams</span>
                        </NuxtLink>
                      </li>-->
                      <li>
                        <button @click="changeTheme">
                          <MoonIcon v-if="$colorMode.value === 'light'" />
                          <SunIcon v-else />
                          <span v-if="$colorMode.value === 'light'">
                            Dark Mode</span
                          >
                          <span v-else>Light Mode</span>
                        </button>
                      </li>
                      <hr />
                      <li>
                        <button @click="logout">
                          <LogOutIcon />
                          <span>Log Out</span>
                        </button>
                      </li>
                    </ul>
                  </div>
                </div>
              </section>
            </template>
            <template v-else>
              <section class="auth-prompt">
                <a :href="authUrl" class="log-in-button"
                  ><GitHubIcon aria-hidden="true" />Sign In with GitHub</a
                >
              </section>
            </template>
          </client-only>
        </section>
      </section>
    </header>
    <main>
      <notifications group="main" position="bottom right" />
      <!--<notifications
        group="ads"
        position="bottom right"
        :duration="-1"
        :ignore-duplicates="true"
      />-->
      <Nuxt />
    </main>
  </div>
</template>

<script>
import ClickOutside from 'vue-click-outside'

import ModrinthLogo from '~/assets/images/text-logo.svg?inline'
import ModrinthLogoWhite from '~/assets/images/text-logo-white.svg?inline'

import NotificationIcon from '~/assets/images/sidebar/notifications.svg?inline'

import DropdownIcon from '~/assets/images/utils/dropdown.svg?inline'
import MoonIcon from '~/assets/images/utils/moon.svg?inline'
import SunIcon from '~/assets/images/utils/sun.svg?inline'

import UserIcon from '~/assets/images/utils/user.svg?inline'
import LogOutIcon from '~/assets/images/utils/log-out.svg?inline'
import GitHubIcon from '~/assets/images/utils/github.svg?inline'

export default {
  components: {
    ModrinthLogo,
    ModrinthLogoWhite,
    DropdownIcon,
    MoonIcon,
    SunIcon,
    UserIcon,
    LogOutIcon,
    GitHubIcon,
    NotificationIcon,
  },
  directives: {
    ClickOutside,
  },
  async fetch() {
    if (this.$route.query.code) {
      await this.$auth.setUserToken(this.$route.query.code)
    }
  },
  data() {
    return {
      isDropdownOpen: false,
    }
  },
  computed: {
    authUrl() {
      return `https://api.modrinth.com/api/v1/auth/init?url=http://modrinth.com${this.$route.path}`
    },
    userUrl() {
      return `/user/${this.$auth.user.id}`
    },
    userTeamsUrl() {
      return `${this.userUrl}/teams`
    },
  },
  methods: {
    toggleDropdown() {
      this.isDropdownOpen = !this.isDropdownOpen
    },
    hideDropdown() {
      this.isDropdownOpen = false
    },
    logout() {
      this.$auth.setToken('local', false)
      this.$router.go(null)
    },
    changeTheme() {
      this.$colorMode.preference =
        this.$colorMode.value === 'dark' ? 'light' : 'dark'
    },
  },
}
</script>

<style lang="scss">
.layout {
  background-color: var(--color-bg);
  display: block;
  height: 100vh;

  .site-header {
    height: var(--size-navbar-height);
    background-color: var(--color-raised-bg);
    max-width: 100vw;
    .navbar {
      margin: 0 var(--spacing-card-lg);
      section.logo {
        align-items: center;
        display: flex;
        justify-content: space-between;
        padding: 1rem 0;
        color: var(--color-text-dark);
        svg {
          height: 1.75rem;
          width: auto;
        }
        .badge {
          margin-left: 0.25rem;
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
      section.nav {
        .tabs {
          position: relative;
          top: 50%;
          transform: translateY(-50%);
          .section {
            border-left: 3px solid var(--color-brand-disabled);
            margin-left: 0.75rem;
            padding-left: 0.75rem;
          }
          .tab {
            font-size: var(--font-size-md);
          }
        }
      }
      section.user-controls {
        align-items: center;
        display: flex;
        justify-content: space-between;
        position: relative;
        top: 50%;
        transform: translateY(-50%);
        min-width: 12rem;
        .dropdown {
          position: relative;
          display: inline-block;
          flex-grow: 1;
          &:hover .control {
            border-radius: var(--size-rounded-control);
            background: var(--color-button-bg);
          }
          &.open {
            .control {
              background: var(--color-button-bg);
              border-radius: var(--size-rounded-control)
                var(--size-rounded-control) 0 0;
              .dropdown-icon {
                transform: rotate(180deg);
              }
            }
            .content {
              display: unset;
            }
          }
          .control {
            border-radius: var(--size-rounded-control);
            align-items: center;
            display: flex;
            padding: 0.3rem 0.75rem;
            position: relative;
            z-index: 10;
            width: 100%;
            .avatar {
              align-items: center;
              display: flex;
              flex-grow: 1;
              .icon {
                border-radius: 50%;
                height: 2rem;
                width: 2rem;
                margin-left: 0.5rem;
                margin-right: 0.25rem;
              }
              span {
                display: block;
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
                color: var(--color-text-dark);
                font-weight: var(--font-weight-medium);
              }
            }
            .dropdown-icon {
              color: var(--color-text-dark);
              transition: 150ms ease transform;
            }
          }
          .content {
            margin: 0rem 0 0 0rem;
            min-width: 10rem;
            width: 100%;
            position: fixed;
            display: none;
          }
          button {
            background-color: transparent;
            color: var(--color-text-dark);
            margin: 0;
            padding: 0;
            font-weight: var(--font-weight-medium);
          }
          ul {
            background-color: var(--color-button-bg);
            border-radius: 0 0 var(--size-rounded-control)
              var(--size-rounded-control);
            box-shadow: var(--shadow-dropdown);
            display: flex;
            flex-direction: column;
            margin: 0;
            list-style: none;
            padding: 0.5rem 0;
            z-index: 1;
            hr {
              background-color: var(--color-divider-dark);
              border: none;
              color: var(--color-divider-dark);
              height: 2px;
              margin: 0.5rem 0;
            }
            li {
              margin: 0;
              &:hover,
              &:focus,
              &:active {
                background-color: var(--color-button-bg-active);
                color: var(--color-text-dark);
              }
              a,
              button {
                align-items: center;
                display: flex;
                padding: 0.75rem 1.5rem;
                color: var(--color-text-dark);
                svg {
                  color: inherit;
                  height: 1rem;
                  width: 1rem;
                }
                span {
                  margin-left: 0.5rem;
                }
              }
            }
          }
        }
      }
      section.auth-prompt {
        display: flex;
        align-items: center;
        height: 100%;
        .log-in-button {
          text-align: center;
          border-radius: var(--size-rounded-control);
          background-color: var(--color-brand);
          white-space: nowrap;
          outline: none;
          color: var(--color-brand-inverted);
          display: block;
          padding: 0.5rem 0.75rem;
          svg {
            vertical-align: middle;
            margin-right: 0.5rem;
          }
          &:hover,
          &:focus {
            background-color: var(--color-brand-2);
          }
        }
      }
      @media screen and (min-width: 1024px) {
        max-width: 1280px;
        margin-left: auto;
        margin-right: auto;
      }
    }
  }

  main {
    grid-area: 'main';
  }
}

@media (min-width: 1024px) {
  .layout {
    header {
    }

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

@media (min-width: 1280px) {
  .layout {
    header {
    }
  }
}
</style>
