<template>
  <div class="page">
    <div class="cover">
      <img
        class="cover-image"
        src="~/assets/images/landing.svg"
        width="100%"
        alt="cover-image"
      />
      <div class="text">
        <ModrinthLogo aria-hidden="true" class="modrinth-logo" />
        <h1>Discover, Play, and Create Minecraft content</h1>
        <h3>
          Find enjoyable, quality content through our
          <a href="https://github.com/modrinth" target="_blank">open-source</a>
          modding platform built for the community. Create stuff, get paid, and
          deploy your project with our
          <a href="https://docs.modrinth.com" target="_blank"
            >fully documented</a
          >
          API!
        </h3>
        <form action="/mods">
          <div class="iconified-input">
            <label class="hidden" for="q">Search Mods</label>
            <SearchIcon />
            <input
              id="q"
              type="search"
              name="q"
              placeholder="Search mods..."
              autocomplete="off"
            />
          </div>
          <button class="iconified-button brand-button" type="submit">
            <RightArrowIcon />
            Search
          </button>
        </form>
      </div>
    </div>
    <div class="points card">
      <div class="point">
        <div class="image">
          <img
            v-if="$colorMode.value === 'light'"
            src="https://cdn.modrinth.com/homepage/search-light.webp"
            alt="Screenshot of Modrinth's 'search mods' page"
          />
          <img
            v-else-if="$colorMode.value === 'oled'"
            src="https://cdn.modrinth.com/homepage/search-oled.webp"
            alt="Screenshot of Modrinth's 'search mods' page"
          />
          <img
            v-else
            src="https://cdn.modrinth.com/homepage/search-dark.webp"
            alt="Screenshot of Modrinth's 'search mods' page"
          />
        </div>
        <div class="text-container">
          <h3 class="subheader">Fast and powerful search</h3>
          <h1>Find what you want, quickly and easily</h1>
          <p>
            We are invested in making it extremely simple to find what you want
            to play. Our search is adaptable, customizable, and easy to use!
            Whether you're a power user or someone who just discovered the world
            of Minecraft modding, we've got you covered.
          </p>
        </div>
      </div>
      <div class="point left">
        <div class="text-container">
          <div>
            <h3 class="subheader">Packed with features</h3>
            <h1>Constantly evolving and improving</h1>
            <p>
              We are always adding new features and working towards making
              Modrinth have the best possible user experience. Right now, we're
              working on giving creators more analytics, adding new types of
              projects, our launcher, and much more! If you have any more
              feature ideas, feel free to join our
              <a href="https://discord.gg/EUHuJHt" target="_blank">Discord</a>!
            </p>
            <div class="features">
              <div class="feature completed">
                <CheckIcon />
                <p>
                  <a href="https://github.com/modrinth" target="_blank">
                    100% open source
                  </a>
                </p>
              </div>
              <div class="feature completed">
                <CheckIcon />
                <p>Real-time search</p>
              </div>
              <div class="feature completed">
                <CheckIcon />
                <p>
                  <a href="https://docs.modrinth.com" target="_blank">
                    Completely documented API
                  </a>
                </p>
              </div>
              <div class="feature completed">
                <CheckIcon />
                <p>Dependency management</p>
              </div>
              <div class="feature completed">
                <CheckIcon />
                <p>Modpacks</p>
              </div>
              <div class="feature completed">
                <CheckIcon />
                <p>Creator monetization</p>
              </div>
              <div class="feature in-progress">
                <InProgressIcon />
                <p>Creator analytics</p>
              </div>
              <div class="feature in-progress blurred">
                <InProgressIcon />
                <p>[Redacted]</p>
              </div>
              <div class="feature in-progress">
                <InProgressIcon />
                <p>More types of projects</p>
              </div>
              <div class="feature in-progress">
                <InProgressIcon />
                <p>Game launcher</p>
              </div>
              <div class="feature in-progress">
                <InProgressIcon />
                <p>In-house authentication</p>
              </div>
              <div class="feature in-progress">
                <InProgressIcon />
                <p>Project comments</p>
              </div>
            </div>
          </div>
        </div>
        <div class="image">
          <FeaturesIllustration />
        </div>
      </div>
      <div class="point">
        <div class="image">
          <div class="terminal">
            <div class="line-nums">
              <span>1</span>
              <span>2</span>
              <span>3</span>
              <span>4</span>
              <span>5</span>
              <span>6</span>
              <span>7</span>
              <span>8</span>
              <span>9</span>
              <span>10</span>
              <span>11</span>
              <span>12</span>
              <span>13</span>
              <span>14</span>
              <span>15</span>
            </div>
            <div v-highlightjs class="text">
              <span class="command">curl</span>
              <span>https://api.modrinth.com/v2/project/sodium</span>
              <pre><code class="language-json">
{
  "id": "AANobbMI",
  "slug": "sodium",
  "project_type": "mod",
  "team": "4reLOAKe",
  "title": "Sodium",
  "description": "Modern rendering engine and client-side optimization mod for Minecraft",
  "body": "...",
  "body_url": "https://cdn.modrinth.com/data/AANobbMI/description.md",
  "published": "2021-01-03T00:53:34.185936Z",
  "updated": "2022-01-14T02:35:40.373686Z",
  ...
}</code></pre>
            </div>
          </div>
        </div>
        <div class="text-container">
          <h3 class="subheader">Simple to integrate</h3>
          <h1>An open-source API for everyone to use</h1>
          <p>
            We have created an open source modding platform for the community.
            We're committed to open source so the community can trust us as
            their modding platform. Our API is
            <a href="https://docs.modrinth.com" target="_blank"
              >fully documented</a
            >
            so anyone can use it. We will never add restrictions on our API!
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import SearchIcon from '~/assets/images/utils/search.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'
import InProgressIcon from '~/assets/images/utils/updated.svg?inline'
import RightArrowIcon from '~/assets/images/utils/right-arrow.svg?inline'

import FeaturesIllustration from '~/assets/images/illustrations/features.svg?inline'
import ModrinthLogo from '~/assets/images/text-logo.svg?inline'

export default {
  components: {
    SearchIcon,
    RightArrowIcon,
    CheckIcon,
    InProgressIcon,
    FeaturesIllustration,
    ModrinthLogo,
  },
  auth: false,
  data() {
    return {}
  },
  methods: {},
}
</script>

<style lang="scss" scoped>
.page {
  max-width: 1280px;
  margin: 0 auto;

  .cover {
    img {
      border-radius: var(--size-rounded-card);
      width: 100%;
      height: 40rem;
      object-fit: cover;
      object-position: 15% 12.5%;
      box-shadow: var(--shadow-card);

      @media screen and (max-width: 1280px) {
        border-radius: 0;
      }
    }

    .text {
      position: absolute;
      top: calc(8rem + var(--size-navbar-height));
      max-width: 30rem;
      text-shadow: 4px 4px 8px rgba(0, 0, 0, 0.3);
      padding-left: 6rem;

      color: #fff;

      .modrinth-logo {
        display: none;
      }

      h1 {
        color: #fff;
        margin: 1rem 0;
        font-size: var(--font-size-2xl);
      }

      h3 {
        color: #fff;
        font-size: var(--font-size-lg);
        font-weight: normal;
        margin-bottom: 1rem;

        a {
          text-decoration: underline;
        }
      }

      form {
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 0.25rem;
        box-sizing: border-box;

        button {
          max-height: unset;
          min-height: 40px;
        }
      }

      @media screen and (max-width: 750px) {
        padding: 1rem 1.5rem;
        top: 5rem;

        .modrinth-logo {
          --color-brand: currentColor;
          display: unset;
          width: 14rem;
          max-width: 100%;
          height: auto;
        }
      }
    }
  }

  .points {
    border-radius: 0;
    padding: var(--spacing-card-lg);
    margin-top: 1rem;

    .point {
      display: flex;
      align-items: center;
      flex-direction: column-reverse;
      margin-bottom: 1rem;

      &.left {
        flex-direction: column;
      }

      .text-container {
        .subheader {
          margin: 0;
          color: var(--color-brand);
        }

        a {
          text-decoration: underline;
        }

        .features {
          display: flex;
          flex-wrap: wrap;
          row-gap: 1rem;

          .feature {
            width: 100%;
            display: flex;
            align-items: flex-start;

            p {
              margin: 0;
              line-height: 1.5rem;
            }

            svg {
              min-width: 1.5rem;
              height: 1.5rem;
              margin-right: 0.5rem;
            }

            &.completed {
              svg {
                color: var(--color-brand);
              }
            }

            &.in-progress {
              svg {
                color: var(--color-badge-yellow-bg);
              }
            }

            &.blurred {
              p {
                user-select: none;
                color: transparent;
                text-shadow: 0 0 12px var(--color-badge-yellow-bg);
              }
            }
          }
        }
      }

      .image {
        display: none;
        padding: 2rem;

        svg {
          color: var(--color-brand);
          width: 36rem;
          height: unset;
        }

        img {
          border-radius: var(--size-rounded-lg);
          box-shadow: var(--shadow-floating);
          width: 40rem;
        }

        .terminal {
          display: flex;
          padding: 0.5rem 1rem;
          border-radius: var(--size-rounded-lg);
          background-color: var(--color-code-bg);
          font-family: monospace;
          font-size: 80%;
          line-height: 1.5rem;
          box-shadow: var(--shadow-floating);

          .line-nums {
            display: flex;
            flex-direction: column;
            padding: 0.5rem;
            color: var(--color-icon);
          }

          .text {
            padding: 0.5rem;
            .command {
              color: var(--color-brand);
            }

            pre {
              margin: 0;
              padding: 0;
              border-radius: 0;
              overflow-y: hidden;
              background-color: var(--color-code-bg);

              code {
                padding: 0;

                border-radius: 0;
                background-color: var(--color-code-bg);
                color: var(--color-code-text);
              }
            }
          }
        }
      }
    }
  }

  @media screen and (min-width: 500px) {
    .points {
      .point {
        .text-container {
          .features {
            .feature {
              width: 50%;
            }
          }
        }
      }
    }
  }

  @media screen and (min-width: 750px) {
    .points {
      .point {
        margin-bottom: 0;

        .image {
          display: block;
        }
      }
    }
  }

  @media screen and (min-width: 1200px) {
    .points {
      border-radius: var(--size-rounded-card);
      .point {
        flex-direction: row;

        &.left {
          flex-direction: row;

          .text-container {
            padding-left: 3rem;
            padding-right: 0;
          }
        }

        .text-container {
          width: 60%;
          padding-right: 3rem;

          .features {
            .feature {
              width: 33%;
            }
          }
        }
      }
    }
  }
}
</style>
