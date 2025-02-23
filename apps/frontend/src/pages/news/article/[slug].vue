<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import {
  CheckIcon,
  RssIcon,
  NewspaperIcon,
  BlueskyIcon,
  TwitterIcon,
  MastodonIcon,
  LinkIcon,
  MailIcon,
} from "@modrinth/assets";
import dayjs from "dayjs";
import { ref } from "vue";
import { renderHighlightedString } from "@modrinth/utils";

const article = {
  title: "Host your own server with Modrinth Servers — now in beta",
  short_title: "Introducing Modrinth Servers",
  summary: "Fast, simple, reliable servers directly integrated into Modrinth.",
  short_summary: "Host your next Minecraft server with Modrinth.",
  thumbnail:
    "https://media.beehiiv.com/cdn-cgi/image/format=auto,width=800,height=421,fit=scale-down,onerror=redirect/uploads/asset/file/eefddc59-b4c4-4e7d-92e8-c26bdef42984/Modrinth-Servers-Thumb.png",
  date: dayjs("2024-11-02T22:00:00-08:00"),
  slug: "modrinth-servers-beta",
  body: `It's been almost *four* years since we publicly launched Modrinth Beta. Today, we're thrilled to unveil a new beta release of a product we've been eagerly developing: Modrinth Servers.

Modrinth Servers aims to provide the most seamless experience for running and playing on modded servers. To make this possible, we have partnered with our friends at the server hosting provider [Pyro](https://pyro.host). Together, we've developed fully custom software that gives us a unique advantage in scaling, offering new features and integrations that other hosts couldn't dream of.

For this beta launch, **all servers are US-only**. Please be aware of this if you are looking to purchase a server, as it may not be optimal for users outside of North America.

![A screenshot of the fully-custom Modrinth Servers panel integrated into Modrinth](https://lh7-rt.googleusercontent.com/docsz/AD_4nXdMtvk3YM01h-1NdDrHwM9loR0jIT0wMIrXhmmlEKqjuSzxs4jNnIpdAUBC2ajMehPWdPfIJIKyM3WZIeUauy1UM22xByQqGzhJbRDKoB-XepB0SMFAI8xcRkDpBxrjnBwweqJXygxmjSJxBsxTduH5Lv00?key=Xs_3YjsnFxbZJn1po6YXKPs5)

## What makes Modrinth Servers unique?
We understand that entering the server hosting industry might come as a surprise given the number of existing providers. Here's what sets Modrinth Servers apart:

### The most modern hardware
Your modpack shouldn't have to run slow. All our servers are powered by cutting-edge 2023 Ryzen 7 and Ryzen 9 CPUs with DDR5 memory. From our research, we couldn't find any other Minecraft server host offering such modern hardware at any price point, much less at our affordably low one. This ensures smooth performance even with the most demanding modpacks.

### Seamless integration with Modrinth content
Download mods and modpacks directly from Modrinth without any hassle. This deep integration simplifies server setup and management like never before. With just a few clicks, you can have your server up and running with your favorite mods.

### Fully custom panel and backend
Unlike most other server hosts that rely on off-the-shelf software like Multicraft or Pterodactyl, Modrinth Servers is fully custom-built from front to back. This enables higher performance and much deeper integration than is otherwise possible. Our intuitive interface makes server management a breeze, even for newcomers.

### Dedicated support
Our team is committed to providing exceptional support. Whether you're experiencing technical issues or have questions, we're here to ensure your experience with Modrinth Servers is top-notch.

### No tricky fees or up-charges
Modrinth Servers are offered in a very simple Small, Medium, and Large pricing model, and are priced based on the amount of RAM at $3/GB. Custom URLs, port configuration, off-site backups, and plenty of storage is included in every Modrinth Server purchase at no additional cost.

## What’s next?
As this is a beta release, there's much more to come for Modrinth Servers:
- **Global availability:** We plan to expand to more worldwide regions and offer the ability to select a region for your server, ensuring optimal performance no matter where you are.
- **Support more types of content:** We'll be adding support for plugin loaders and improving support for data packs, giving you more flexibility and functionality
- **Social features:** A friends system to make sharing invites to servers easier, streamlining sharing custom-built modpacks and servers with your community.
- **App integration:** Full integration with Modrinth App, including the ability to sync an instance with a server or friends, making collaboration seamless.
- **Collaborative management:** Give other Modrinth users access to your server panel so you can manage your server with your team.
- **Automatic creator commissions:** Creators will automatically earn a portion of server proceeds when content is installed on a Modrinth Server.

And so much more... stay tuned!

We can't wait for you to try out [Modrinth Servers](https://modrinth.gg) and share your feedback. This is just the beginning, and we're excited to continue improving and expanding our services to better serve the Minecraft community.

**From the teams at Modrinth and Pyro, with <3**`,
};

const articleUrl = encodeURIComponent(`https://modrinth.com/news/article/${article.slug}`);

const copied = ref(false);

async function copyToClipboard(text: string) {
  await navigator.clipboard.writeText(text);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 3000);
}
</script>

<template>
  <div class="page experimental-styles-within">
    <div
      class="flex items-center justify-between gap-6 border-0 border-b-[1px] border-solid border-divider pb-6"
    >
      <nuxt-link :to="`/news`">
        <h1 class="m-0 text-3xl font-extrabold hover:underline">News</h1>
      </nuxt-link>
      <div class="flex gap-2">
        <ButtonStyled color="brand" type="outlined">
          <button><NewspaperIcon /> Sign up for our newsletter</button>
        </ButtonStyled>
        <ButtonStyled circular>
          <button v-tooltip="`RSS feed`" aria-label="RSS feed">
            <RssIcon />
          </button>
        </ButtonStyled>
      </div>
    </div>
    <article class="mt-6 flex flex-col gap-4">
      <h2 class="m-0 text-4xl font-extrabold">{{ article.title }}</h2>
      <p class="m-0 text-lg leading-tight">{{ article.summary }}</p>
      <div class="mt-auto text-secondary">Posted on {{ article.date.format("MMMM D, YYYY") }}</div>
      <div class="flex gap-2">
        <ButtonStyled circular>
          <a
            v-tooltip="`Share on Bluesky`"
            :href="`https://bsky.app/intent/compose?text=${articleUrl}`"
            target="_blank"
          >
            <BlueskyIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <a
            v-tooltip="`Share on Mastodon`"
            :href="`https://tootpick.org/#text=${articleUrl}`"
            target="_blank"
          >
            <MastodonIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <a
            v-tooltip="`Share on X`"
            :href="`https://www.x.com/intent/post?url=${articleUrl}`"
            target="_blank"
          >
            <TwitterIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <a
            v-tooltip="`Share via email`"
            :href="`mailto:?subject=${encodeURIComponent(article.title)}&body=${articleUrl}`"
            target="_blank"
          >
            <MailIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <button
            v-tooltip="copied ? `Copied to clipboard` : `Copy link`"
            :disabled="copied"
            @click="copyToClipboard(articleUrl)"
          >
            <CheckIcon v-if="copied" />
            <LinkIcon v-else />
          </button>
        </ButtonStyled>
      </div>
      <img
        :src="article.thumbnail"
        class="aspect-video w-full rounded-2xl border-[1px] border-solid border-button-border object-cover"
        :alt="article.title"
      />
      <div class="markdown-body" v-html="renderHighlightedString(article.body)" />
    </article>
  </div>
</template>
<style lang="scss" scoped>
.page {
  > *:not(.full-width-bg),
  > .full-width-bg > * {
    max-width: 56rem;
    margin-inline: auto;
  }
}

.brand-gradient-bg {
  background: var(--brand-gradient-bg);
  border-color: var(--brand-gradient-border);
}

:deep(.markdown-body) {
  > *:not(img, :has(img:first-child:last-child)) {
    margin-inline: 2rem;
  }

  h1,
  h2 {
    border-bottom: none;
    padding: 0;
  }

  ul > li:not(:last-child) {
    margin-bottom: 0.5rem;
  }

  ul {
    strong {
      color: var(--color-contrast);
    }
  }

  h1,
  h2,
  h3 {
    margin-bottom: 0.25rem;
  }

  p {
    margin-bottom: 1.25rem;
  }

  a {
    color: var(--color-brand);
    font-weight: 600;

    &:hover {
      text-decoration: underline;
    }
  }

  img {
    border: 1px solid var(--color-button-border);
    border-radius: var(--radius-lg);
  }
}
</style>
