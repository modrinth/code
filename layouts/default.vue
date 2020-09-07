<template>
  <div class="layout">
    <aside>
      <div class="logo-wrapper">
        <img class="logo" src="~/assets/images/logo.svg" />
        <span class="name">modrinth</span>
      </div>
      <nav>
        <section class="links">
          <h3>Projects</h3>
          <section>
            <nuxt-link to="/modpacks">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <line x1="16.5" y1="9.4" x2="7.5" y2="4.21" />
                <path
                  d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
                />
                <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
                <line x1="12" y1="22.08" x2="12" y2="12" />
              </svg>
              <span> Modpacks </span>
            </nuxt-link>
            <nuxt-link to="/mods">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="16 18 22 12 16 6" />
                <polyline points="8 6 2 12 8 18" />
              </svg>
              <span>Mods</span>
            </nuxt-link>
          </section>

          <h3>Community</h3>
          <section>
            <nuxt-link to="/support">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="12" cy="12" r="10" />
                <circle cx="12" cy="12" r="4" />
                <line x1="4.93" y1="4.93" x2="9.17" y2="9.17" />
                <line x1="14.83" y1="14.83" x2="19.07" y2="19.07" />
                <line x1="14.83" y1="9.17" x2="18.36" y2="5.64" />
                <line x1="4.93" y1="19.07" x2="9.17" y2="14.83" />
              </svg>
              <span>Support</span>
            </nuxt-link>
            <nuxt-link to="/guides">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
                <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
              </svg>
              <span>Guides</span>
            </nuxt-link>
          </section>

          <h3>Dashboard</h3>
          <section>
            <nuxt-link to="/dashboard/projects">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path
                  d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                />
              </svg>
              <span>My projects</span>
            </nuxt-link>
            <nuxt-link to="/dashboard/analytics">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
              </svg>
              <span>Analytics</span>
            </nuxt-link>
          </section>
        </section>

        <section class="user-actions">
          <div class="avatar">
            <img src="~/assets/images/avatar.jpg" />
            <span> falseresync </span>
          </div>
          <div class="notifications">
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
              <path d="M13.73 21a2 2 0 0 1-3.46 0" />
            </svg>
          </div>
        </section>
      </nav>
    </aside>
    <main>
      <!-- <header>
        <div class="search-wrapper">
          <input type="search" name="search" id="search" placeholder="Search...">
          <!/-- Icon follows and not precedes the input so we can target it with CSS' ~ selector --/>
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
        </div>
      </header> -->

      <nuxt />
    </main>
  </div>
</template>

<style lang="scss">
.layout {
  display: flex;
  min-height: 100vh;
  width: 100%;

  aside {
    top: 0;
    position: -webkit-sticky;
    position: sticky;
    max-height: 100vh;
    border-right: 1px solid var(--color-grey-2);
    display: flex; // Flex here to safely expand navigation height
    flex-direction: column;
    min-width: 15%;

    .logo-wrapper {
      align-items: center;
      display: flex;
      height: 3.5rem;
      padding: 0 1.5rem;
      font-family: 'Montserrat', sans-serif;

      .logo {
        height: 2rem;
        width: auto;
      }

      .name {
        font-family: 'Montserrat Alternates';
        margin-left: 0.4rem;
        font-size: 1.3rem;
      }
    }

    nav {
      display: flex;
      flex-direction: column;
      flex-grow: 1;
      justify-content: space-between;

      & > * {
        padding: 0 1.5rem;
      }

      .links {
        h3 {
          color: #718096;
          font-size: 0.8rem;
          letter-spacing: 0.02rem;
          margin-bottom: 0.5rem;
          margin-top: 1.5rem;
          text-transform: uppercase;
        }

        section {
          border-left: 4px solid var(--color-grey-3);
          margin-left: 0.5rem;

          a {
            align-items: center;
            border-radius: 0 0.25rem 0.25rem 0;
            color: var(--color-grey-5);
            display: flex;
            margin-bottom: 0.25rem;
            padding: 0.5rem 1rem;

            &:hover,
            &:focus,
            &.nuxt-link-active {
              background-color: var(--color-grey-1);
              color: var(--color-text);
            }

            &.nuxt-link-active {
              box-shadow: -4px 0 0 0 var(--color-brand);
            }

            svg {
              height: 1rem;
              width: 1rem;
            }

            span {
              margin-left: 0.5rem;
            }
          }
        }
      }
      .user-actions {
        align-items: center;
        border-top: 2px solid var(--color-grey-2);
        display: flex;
        justify-content: space-between;
        margin-top: 1rem;
        padding-bottom: 1rem;
        padding-top: 1rem;

        & > * {
          align-items: center;
          display: flex;
        }

        svg {
          color: var(--color-grey-5);

          &:hover,
          &:focus {
            color: inherit;
          }
        }

        .avatar {
          img {
            border-radius: 50%;
            height: 2rem;
            margin-right: 0.5rem;
            width: 2rem;
          }
        }
      }
    }
  }
  main {
    background-color: var(--color-grey-0);
    flex-grow: 1;

    header {
      align-items: center;
      background-color: var(--color-bg);
      box-shadow: 0 1px 1px 0 var(--color-grey-2);
      display: flex;
      height: 3.5rem;
      justify-content: space-between;
      padding: 0 3rem 0 1rem;

      .search-wrapper {
        align-items: center;
        display: flex;
        flex-direction: row-reverse;
        width: 100%;

        input {
          border: none;
          font-size: 1rem;
          padding: 1rem;
          width: 100%;

          &::placeholder {
            color: var(--color-grey-5);
          }

          &:hover,
          &:focus {
            & + svg {
              color: inherit;
            }

            &::placeholder {
              color: var(--color-grey-7);
            }
          }
        }

        svg {
          color: var(--color-grey-5);
        }
      }
    }

    .content {
      padding: 1rem 3rem 1rem 2rem;
    }
  }
}
</style>
