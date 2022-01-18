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
        <h1>Discover, Play, and Create Minecraft content</h1>
        <h3>
          Find enjoyable and quality content through our open-source modding
          platform built for the community. Create stuff, get paid*, and deploy
          your project with our fully documented API!
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
          <button class="iconified-button brand-button-colors" type="submit">
            <RightArrowIcon />
            Search
          </button>
        </form>
      </div>
    </div>
  </div>
</template>

<script>
import SearchIcon from '~/assets/images/utils/search.svg?inline'
import RightArrowIcon from '~/assets/images/utils/right-arrow.svg?inline'

export default {
  components: {
    SearchIcon,
    RightArrowIcon,
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
      height: calc(75vh - var(--size-navbar-height));
      object-fit: cover;
      object-position: 10% 12.5%;

      @media screen and (max-width: 1280px) {
        border-radius: 0;
      }

      @media screen and (max-width: 750px) {
        height: calc(75vh);
      }
    }

    .text {
      position: absolute;
      top: calc(10vh + var(--size-navbar-height));
      max-width: 30rem;
      text-shadow: 4px 4px 8px rgba(0, 0, 0, 0.3);
      padding-left: 6rem;

      color: #fff;

      h2 {
        color: #fff;
        text-transform: uppercase;
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
      }

      form {
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 0.25rem;
        box-sizing: border-box;

        input {
          box-sizing: content-box;
        }

        .iconified-button {
          padding: 1.25rem 1rem;
        }
      }

      @media screen and (max-width: 750px) {
        padding-left: 3rem;
        max-width: 55%;
      }
    }
  }
}
</style>
