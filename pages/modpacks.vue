<template>
  <div class="content">
    <h2>Modpacks</h2>
    <div class="columns">
      <div class="column-grow-4">
        <section id="search-pagination">
          <div class="iconified-input column-grow-2">
            <input
              id="search"
              type="search"
              name="search"
              placeholder="Search modpacks"
            />
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
          </div>
          <div class="pagination column-grow-1">pagination</div>
        </section>
      </div>
      <section class="column-grow-1">
        <h3>Filters</h3>
      </section>
    </div>
  </div>
</template>

<script>
export default {
  auth: false,
  head: {
    title: 'Packs - Modrinth',
    meta: [
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Packs',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Packs',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/modpacks`,
      },
    ],
  },
}
</script>

<style lang="scss">
#search-pagination {
  align-items: center;
  display: flex;
  justify-content: space-between;
}
</style>
