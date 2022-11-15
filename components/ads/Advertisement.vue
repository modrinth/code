<template>
  <div class="content-wrapper">
    <div data-ea-publisher="modrinth-com" data-ea-type="text" class="loaded">
      <div class="ea-placement ea-type-text">
        <div class="ea-content">
          <div class="ea-text">
            <a
              href="https://exaroton.com/?utm_source=modrinth&utm_medium=text&utm_campaign=host&utm_content=top"
              rel="nofollow noopener"
              target="_blank"
            >
              <LightIcon
                v-if="$colorMode.value === 'light'"
                class="brand-icon"
              />
              <DarkIcon v-else class="brand-icon" />
              <span>
                <span> Host your Minecraft server on </span>
                <strong>exaroton</strong>
                <span>
                  - only pay while the server is running - billed per second.
                </span>
              </span>
            </a>
          </div>
        </div>
        <div class="ea-callout">
          <a
            rel="nofollow noopener"
            target="_blank"
            href="https://modrinth.com"
          >
            Ads via Adrinth
          </a>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import LightIcon from '~/assets/images/external/exaroton-light.svg?inline'
import DarkIcon from '~/assets/images/external/exaroton-dark.svg?inline'

export default {
  name: 'Advertisement',
  components: {
    LightIcon,
    DarkIcon,
  },
  data() {
    return {
      isBlocked: true,
    }
  },
}
</script>

<style lang="scss">
.content-wrapper {
  min-height: 46.2px;
  margin-bottom: var(--spacing-card-md);

  background: var(--color-ad) !important;
  border: 3px solid var(--color-ad-raised) !important;
  border-radius: var(--size-rounded-card) !important;
}

.loaded {
  position: relative;
}

.ea-content {
  color: var(--color-text) !important;

  box-shadow: none !important;
  background: none !important;
  border-radius: 0 !important;
  margin: 0 !important;

  padding: 1em;
  text-align: left;
}

.brand-icon {
  padding-top: 1px;
  height: 1.2rem;
  width: auto;
}

.ea-callout {
  position: absolute;
  bottom: -2px;
  right: -2px;
  text-align: center;
  font-weight: 600;
  text-transform: uppercase;
  font-size: 0.8em;
  background: var(--color-ad-raised);
  letter-spacing: 0.1ch;

  font-style: normal !important;
  margin: 0 !important;
  padding: 2px 10px !important;
  border-top-left-radius: var(--size-rounded-card);
  border-bottom-right-radius: var(--size-rounded-card);
}

[data-ea-publisher].loaded .ea-callout a,
[data-ea-type].loaded .ea-callout a {
  display: flex;
  align-items: center;
  gap: 0.5rem;

  font-size: 0.8em;

  &:hover {
    color: var(--color-text) !important;
  }

  &:link {
    color: var(--color-text) !important;
  }
}

[data-ea-publisher].loaded .ea-content a,
[data-ea-type].loaded .ea-content a {
  display: flex;
  align-items: center;
  gap: 0.5rem;

  b,
  strong {
    color: #088cdb;
  }

  &:link {
    color: var(--color-text) !important;
  }
}

[data-ea-type='text'].loaded .ea-content,
.ea-type-text .ea-content {
  text-align: left;
}

[data-ea-publisher].loaded .ea-content,
[data-ea-type].loaded .ea-content {
  color: #505050;
}

[data-ea-publisher].loaded,
[data-ea-type].loaded {
  font-size: 14px;
  font-family: var(--font-standard) !important;
  font-weight: normal;
  font-style: normal;
  line-height: 1.3em;
}

@media screen and (max-width: 800px) {
  [data-ea-publisher].loaded .ea-content a,
  [data-ea-type].loaded .ea-content a {
    align-items: flex-start;
    flex-direction: column;
  }

  .ea-text {
    margin-bottom: 0.5rem;
  }
}
</style>
