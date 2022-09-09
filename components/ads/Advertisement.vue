<template>
  <div class="info-wrapper">
    <div v-show="isBlocked" id="info-popup">
      <span>
        <span class="info-popup-wrap">
          <a
            class="info-popup-img"
            href="https://docs.modrinth.com/docs/details/carbon"
          >
            <img
              src="https://cdn.modrinth.com/barrier_fixes.png"
              alt="ads via Carbon"
            />
          </a>
          <a
            href="https://docs.modrinth.com/docs/details/carbon"
            class="info-popup-text"
          >
            Please disable your adblocker. Advertisements support this site and
            its creators.
          </a>
        </span>
        <a
          href="https://docs.modrinth.com/docs/details/carbon"
          class="info-popup-poweredby"
          target="_blank"
          rel="noopener sponsored"
          >ads via Carbon
        </a>
      </span>
    </div>
    <client-only>
      <script
        id="_carbonads_js"
        async
        type="text/javascript"
        src="//cdn.carbonads.com/carbon.js?serve=CEAIKK3N&placement=modrinthcom"
      ></script>
    </client-only>
  </div>
</template>

<script>
export default {
  name: 'Advertisement',
  data() {
    return {
      isBlocked: false,
    }
  },
  mounted() {
    setTimeout(() => (this.isBlocked = true), 1000)
  },
}
</script>

<style lang="scss">
.do-not-style {
  position: absolute !important;

  padding: 0 !important;
  margin: 0 !important;

  top: 0 !important;
  left: 0 !important;
  min-height: 2px !important;
  height: 2px !important;
  max-height: 2px !important;

  min-width: 2px !important;
  width: 2px !important;
  max-width: 2px !important;

  background: none !important;
  border: none !important;
}

.info-wrapper {
  min-height: 122px;
  color: var(--color-text);
  background: var(--color-ad);
  margin-bottom: var(--spacing-card-md);
  border-radius: var(--size-rounded-card);

  border: 3px solid var(--color-ad-raised);
  box-sizing: border-box;
  -moz-box-sizing: border-box;
  -webkit-box-sizing: border-box;

  position: relative;

  div:first-child:nth-last-child(4) {
    display: none;
  }
  div:first-child:nth-last-child(3) {
    display: none;
  }
}

#carbonads_1 {
  display: none;
}

#carbonads,
#info-popup {
  overflow: hidden;
  max-width: 100%;

  font-size: 22px;
  box-sizing: content-box;

  padding: var(--spacing-card-sm) var(--spacing-card-sm);

  > span {
    display: block;
  }

  a {
    color: inherit;
    text-decoration: none;

    &:hover {
      color: inherit;
    }
  }
}

.carbon-wrap,
.info-popup-wrap {
  display: flex;
  align-items: center;
}

.carbon-img,
.info-popup-img {
  display: block;
  margin: 0;
  line-height: 1;

  img {
    display: block;
    height: 100px;
    width: auto;
    border-radius: var(--size-rounded-card);
  }
}

.carbon-text,
.info-popup-text {
  display: block;
  padding: 0 1em;
  line-height: 1.35;
  text-align: left;
}

.carbon-poweredby,
.info-popup-poweredby {
  display: block;
  padding: 8px 10px;
  background: var(--color-ad-raised);
  text-align: center;
  text-transform: uppercase;
  letter-spacing: 0.1ch;
  font-weight: 600;
  font-size: 0.5em;
  line-height: 1;
  border-top-left-radius: var(--size-rounded-card);
  position: absolute;
  bottom: -3px;
  right: -3px;
  border-bottom-right-radius: var(--size-rounded-card);
}

@media only screen and (max-width: 759px) {
  .carbon-text,
  .info-popup-text {
    font-size: 14px;
  }
}
</style>
