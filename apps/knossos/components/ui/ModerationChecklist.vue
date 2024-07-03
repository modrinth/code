<template>
  <div class="card moderation-checklist">
    <h1>Moderation checklist</h1>
    <div v-if="done">
      <p>You are done moderating this project! There are {{ futureProjects.length }} left.</p>
    </div>
    <div v-else-if="generatedMessage">
      <p>
        Enter your moderation message here. Remember to check the Moderation tab to answer any
        questions an author might have!
      </p>
      <div class="markdown-editor-spacing">
        <MarkdownEditor v-model="message" :placeholder="'Enter moderation message'" />
      </div>
    </div>
    <div v-else-if="steps[currentStepIndex].id === 'modpack-permissions'">
      <h2 v-if="modPackData">
        Modpack permissions
        <template v-if="modPackIndex + 1 <= modPackData.length">
          ({{ modPackIndex + 1 }} / {{ modPackData.length }})
        </template>
      </h2>
      <div v-if="!modPackData">Loading data...</div>
      <div v-else-if="modPackData.length === 0">
        <p>All permissions obtained. You may skip this step!</p>
      </div>
      <div v-else-if="!modPackData[modPackIndex]">
        <p>All permission checks complete!</p>
        <div class="input-group modpack-buttons">
          <button class="btn" @click="modPackIndex -= 1">
            <LeftArrowIcon />
            Previous
          </button>
        </div>
      </div>
      <div v-else>
        <div v-if="modPackData[modPackIndex].type === 'unknown'">
          <p>What is the approval type of {{ modPackData[modPackIndex].file_name }}?</p>
          <div class="input-group">
            <button
              v-for="(option, index) in fileApprovalTypes"
              :key="index"
              class="btn"
              :class="{
                'option-selected': modPackData[modPackIndex].status === option.id,
              }"
              @click="modPackData[modPackIndex].status = option.id"
            >
              {{ option.name }}
            </button>
          </div>
          <template v-if="modPackData[modPackIndex].status !== 'unidentified'">
            <div class="universal-labels"></div>
            <label for="proof">
              <span class="label__title">Proof</span>
            </label>
            <input
              id="proof"
              v-model="modPackData[modPackIndex].proof"
              type="text"
              autocomplete="off"
              placeholder="Enter proof of status..."
            />
            <label for="link">
              <span class="label__title">Link</span>
            </label>
            <input
              id="link"
              v-model="modPackData[modPackIndex].url"
              type="text"
              autocomplete="off"
              placeholder="Enter link of project..."
            />
            <label for="title">
              <span class="label__title">Title</span>
            </label>
            <input
              id="title"
              v-model="modPackData[modPackIndex].title"
              type="text"
              autocomplete="off"
              placeholder="Enter title of project..."
            />
          </template>
        </div>
        <div v-else-if="modPackData[modPackIndex].type === 'flame'">
          <p>
            What is the approval type of {{ modPackData[modPackIndex].title }} (<a
              :href="modPackData[modPackIndex].url"
              target="_blank"
              class="text-link"
              >{{ modPackData[modPackIndex].url }}</a
            >?
          </p>
          <div class="input-group">
            <button
              v-for="(option, index) in fileApprovalTypes"
              :key="index"
              class="btn"
              :class="{
                'option-selected': modPackData[modPackIndex].status === option.id,
              }"
              @click="modPackData[modPackIndex].status = option.id"
            >
              {{ option.name }}
            </button>
          </div>
        </div>
        <div
          v-if="
            ['unidentified', 'no', 'with-attribution'].includes(modPackData[modPackIndex].status)
          "
        >
          <p v-if="modPackData[modPackIndex].status === 'unidentified'">
            Does this project provide identification and permission for
            <strong>{{ modPackData[modPackIndex].file_name }}</strong
            >?
          </p>
          <p v-else-if="modPackData[modPackIndex].status === 'with-attribution'">
            Does this project provide attribution for
            <strong>{{ modPackData[modPackIndex].file_name }}</strong
            >?
          </p>
          <p v-else>
            Does this project provide proof of permission for
            <strong>{{ modPackData[modPackIndex].file_name }}</strong
            >?
          </p>
          <div class="input-group">
            <button
              v-for="(option, index) in filePermissionTypes"
              :key="index"
              class="btn"
              :class="{
                'option-selected': modPackData[modPackIndex].approved === option.id,
              }"
              @click="modPackData[modPackIndex].approved = option.id"
            >
              {{ option.name }}
            </button>
          </div>
        </div>
        <div class="input-group modpack-buttons">
          <button class="btn" :disabled="modPackIndex <= 0" @click="modPackIndex -= 1">
            <LeftArrowIcon />
            Previous
          </button>
          <button
            class="btn btn-blue"
            :disabled="!modPackData[modPackIndex].status"
            @click="modPackIndex += 1"
          >
            <RightArrowIcon />
            Next project
          </button>
        </div>
      </div>
    </div>
    <div v-else>
      <h2>{{ steps[currentStepIndex].question }}</h2>
      <template v-if="steps[currentStepIndex].rules && steps[currentStepIndex].rules.length > 0">
        <strong>Rules guidance:</strong>
        <ul>
          <li v-for="(rule, index) in steps[currentStepIndex].rules" :key="index">
            {{ rule }}
          </li>
        </ul>
      </template>
      <template
        v-if="steps[currentStepIndex].examples && steps[currentStepIndex].examples.length > 0"
      >
        <strong>Examples of what to reject:</strong>
        <ul>
          <li v-for="(example, index) in steps[currentStepIndex].examples" :key="index">
            {{ example }}
          </li>
        </ul>
      </template>
      <template
        v-if="steps[currentStepIndex].exceptions && steps[currentStepIndex].exceptions.length > 0"
      >
        <strong>Exceptions:</strong>
        <ul>
          <li v-for="(exception, index) in steps[currentStepIndex].exceptions" :key="index">
            {{ exception }}
          </li>
        </ul>
      </template>
      <p v-if="steps[currentStepIndex].id === 'title'">
        <strong>Title:</strong> {{ project.title }}
      </p>
      <p v-if="steps[currentStepIndex].id === 'slug'"><strong>Slug:</strong> {{ project.slug }}</p>
      <p v-if="steps[currentStepIndex].id === 'summary'">
        <strong>Summary:</strong> {{ project.description }}
      </p>
      <p v-if="steps[currentStepIndex].id === 'links'">
        <template v-if="project.issues_url">
          <strong>Issues: </strong>
          <a class="text-link" :href="project.issues_url">{{ project.issues_url }}</a> <br />
        </template>
        <template v-if="project.source_url">
          <strong>Source: </strong>
          <a class="text-link" :href="project.source_url">{{ project.source_url }}</a> <br />
        </template>
        <template v-if="project.wiki_url">
          <strong>Wiki: </strong>
          <a class="text-link" :href="project.wiki_url">{{ project.wiki_url }}</a> <br />
        </template>
        <template v-if="project.discord_url">
          <strong>Discord: </strong>
          <a class="text-link" :href="project.discord_url">{{ project.discord_url }}</a>
          <br />
        </template>
        <template v-for="(donation, index) in project.donation_urls" :key="index">
          <strong>{{ donation.platform }}: </strong>
          <a class="text-link" :href="donation.url">{{ donation.url }}</a>
          <br />
        </template>
      </p>
      <p v-if="steps[currentStepIndex].id === 'categories'">
        <strong>Categories:</strong>
        <Categories
          :categories="project.categories.concat(project.additional_categories)"
          :type="project.actualProjectType"
          class="categories"
        />
      </p>
      <p v-if="steps[currentStepIndex].id === 'side-types'">
        <strong>Client side:</strong> {{ project.client_side }} <br />
        <strong>Server side:</strong> {{ project.server_side }}
      </p>
      <div class="options input-group">
        <button
          v-for="(option, index) in steps[currentStepIndex].options"
          :key="index"
          class="btn"
          :class="{
            'option-selected':
              selectedOptions[steps[currentStepIndex].id] &&
              selectedOptions[steps[currentStepIndex].id].find((x) => x.name === option.name),
          }"
          @click="toggleOption(steps[currentStepIndex].id, option)"
        >
          {{ option.name }}
        </button>
      </div>
      <div
        v-if="
          selectedOptions[steps[currentStepIndex].id] &&
          selectedOptions[steps[currentStepIndex].id].length > 0
        "
        class="inputs universal-labels"
      >
        <div
          v-for="(option, index) in selectedOptions[steps[currentStepIndex].id].filter(
            (x) => x.fillers && x.fillers.length > 0
          )"
          :key="index"
        >
          <div v-for="(filler, idx) in option.fillers" :key="idx">
            <label :for="filler.id">
              <span class="label__title">
                {{ filler.question }}
                <span v-if="filler.required" class="required">*</span>
              </span>
            </label>
            <div v-if="filler.large" class="markdown-editor-spacing">
              <MarkdownEditor v-model="filler.value" :placeholder="'Enter moderation message'" />
            </div>
            <input v-else :id="filler.id" v-model="filler.value" type="text" autocomplete="off" />
          </div>
        </div>
      </div>
    </div>
    <div class="input-group modpack-buttons">
      <button v-if="!done" class="btn skip-btn" @click="goToNextProject">
        <ExitIcon />
        <template v-if="futureProjects.length > 0">Skip</template>
        <template v-else>Exit</template>
      </button>
      <button v-if="currentStepIndex > 0" class="btn" @click="previousPage() && !done">
        <LeftArrowIcon /> Previous
      </button>
      <button
        v-if="currentStepIndex < steps.length - 1 && !done"
        class="btn btn-primary"
        @click="nextPage()"
      >
        <RightArrowIcon /> Next
      </button>
      <button
        v-else-if="!generatedMessage"
        class="btn btn-primary"
        :disabled="loadingMessage"
        @click="generateMessage"
      >
        <UpdatedIcon /> Generate message
      </button>
      <template v-if="generatedMessage && !done">
        <button class="btn btn-green" @click="sendMessage(project.requested_status ?? 'approved')">
          <CheckIcon /> Approve
        </button>
        <div class="joined-buttons">
          <button class="btn btn-danger" @click="sendMessage('rejected')">
            <CrossIcon /> Reject
          </button>
          <OverflowMenu
            class="btn btn-danger btn-dropdown-animation icon-only"
            position="top"
            direction="left"
            :options="[
              {
                id: 'withhold',
                color: 'danger',
                action: () => sendMessage('withheld'),
                hoverFilled: true,
              },
            ]"
          >
            <DropdownIcon style="rotate: 180deg" />
            <template #withhold> <EyeOffIcon /> Withhold </template>
          </OverflowMenu>
        </div>
      </template>
      <button v-if="done" class="btn btn-primary next-project" @click="goToNextProject">
        Next project
      </button>
    </div>
  </div>
</template>

<script setup>
import {
  LeftArrowIcon,
  MarkdownEditor,
  RightArrowIcon,
  UpdatedIcon,
  CheckIcon,
  OverflowMenu,
  DropdownIcon,
  XIcon as CrossIcon,
  EyeOffIcon,
  ExitIcon,
} from 'omorphia'
import Categories from '~/components/ui/search/Categories.vue'

const props = defineProps({
  project: {
    type: Object,
    default: null,
  },
  futureProjects: {
    type: Array,
    default: () => [],
  },
  resetProject: {
    type: Function,
    required: true,
    default: () => {},
  },
})

const steps = computed(() =>
  [
    {
      id: 'title',
      question: 'Is this title free of useless information?',
      shown: true,
      rules: [
        'No unnecessary data (mod loaders, game versions, etc)',
        'No emojis / useless text decorators',
      ],
      examples: [
        '✅ NoobMod [1.8+] • Kill all noobs in your world!',
        '[FABRIC] My Optimization Pack',
        '[1.17-1.20.4] LagFixer ⚡️ Best Performance Solution! ⭕ Well optimized ✅ Folia supported! (BETA)',
      ],
      exceptions: [
        'Loaders and/or game versions allowed if this project is a port of another mod. (ex: Gravestones for 1.20)',
        'Loaders allowed if they choose to separate their project into Forge and Fabric variants (discouraged)',
      ],
      options: [
        {
          name: 'Contains useless info',
          resultingMessage: `## Misuse of Title
Per section 5.2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) we ask that you limit the title to just the name of your project. Additional information, such as themes, tags, supported versions or loaders, etc. should be saved for the Summary or Description. When changing your project title, remember to also ensure that your project slug (URL) matches and accurately represents your project.`,
        },
      ],
    },
    {
      id: 'slug',
      question: 'Is the slug accurate and appropriate?',
      shown: true,
      rules: ['Matches title / not misleading (acronyms are OK)'],
      options: [
        {
          name: 'Misused',
          resultingMessage: `## Misuse of Slug
Per section 5.2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) your project slug (URL) must accurately represent your project. `,
        },
      ],
    },
    {
      id: 'summary',
      question: `Is the project's summary sufficient?`,
      shown: true,
      rules: [
        'The summary should provide a brief overview of your project that informs and entices users.',
        `Should not be the exact same as the project's title`,
        'Should not include any markdown formatting.',
      ],
      options: [
        {
          name: 'Insufficient',
          resultingMessage: `## Insufficient Summary
Per section 5.3 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) Your project summary should provide a brief overview of your project that informs and entices users.
This is the first thing most people will see about your mod other than the Logo, so it's important it be accurate, reasonably detailed, and exciting.`,
        },
        {
          name: 'Repeat of title',
          resultingMessage: `## Insufficient Summary
Per section 5.3 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) your Summary can not be the same as your project's Title. Your project summary should provide a brief overview of your project that informs and entices users.
This is the first thing most people will see about your mod other than the Logo, so it's important it be accurate, reasonably detailed, and exciting.`,
        },
        {
          name: 'Formatting',
          resultingMessage: `## Insufficient Summary
Per section 5.3 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) your Summary can not include any extra formatting such as lists, or links. Your project summary should provide a brief overview of your project that informs and entices users.
This is the first thing most people will see about your mod other than the Logo, so it's important it be accurate, reasonably detailed, and exciting.`,
        },
      ],
    },
    {
      id: 'description',
      question: `Is the project's description sufficient?`,
      navigate: `/${props.project.project_type}/${props.project.slug}`,
      shown: true,
      rules: [
        'Should answer what the project specifically does or adds ',
        'Should answer why someone should want to download the project ',
        'Should indicate any other critical information the user must know before downloading',
        'Should be accessible (no fancy characters / non-standard text, no image-only descriptions, must have English component, etc)',
      ],
      options: [
        {
          name: 'Insufficient',
          resultingMessage: `## Insufficient Description
Per section 2.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#general-expectations) your project's Description should clearly inform the reader of the content, purpose, and appeal of your project.
Currently, it looks like there are some missing details.
%EXPLAINER%`,
          fillers: [
            {
              id: 'EXPLAINER',
              question: 'Please elaborate on how the author can improve their description.',
              large: true,
            },
          ],
        },
        {
          name: 'Insufficient (default packs)',
          resultingMessage: `## Insufficient Description
Per section 2.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#general-expectations) your project's Description should clearly inform the reader of the content, purpose, and appeal of your project.
Currently, it looks like there are some missing details.
What does your modpack add? What features does it have? Why would a user want to download it? Be specific!
See descriptions like [Simply Optimized](https://modrinth.com/modpack/sop) or [Aged](https://modrinth.com/modpack/aged) for examples of what a good description looks like.
`,
        },
        {
          name: 'Insufficient (default projects)',
          resultingMessage: `## Insufficient Description
Per section 2.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#general-expectations) your project's Description should clearly inform the reader of the content, purpose, and appeal of your project.
Currently, it looks like there are some missing details.
What does your project add? What features does it have? Why would a user want to download it? Be specific!
See descriptions like [Sodium](https://modrinth.com/mod/sodium) or [LambDynamicLights](https://modrinth.com/mod/lambdynamiclights) for examples of what a good description looks like.
`,
        },
        {
          name: 'Non-english',
          resultingMessage: `## No English Description
Per section 2.2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#accessibility) a project's Summary and Description must be in English, unless meant exclusively for non-English use, such as translations.  You may include your non-English Description if you would like but we ask that you also add an English translation of the Description to your Description page, if you would like to use an online translator to do this, we recommend [DeepL](https://www.deepl.com/translator).`,
        },
        {
          name: 'Unfinished',
          resultingMessage: `## Unfinished Description
It looks like your project Description is still a WIP seeing as %REASON%. Please remember to submit only when ready, as it is important your project meets the requirements of Section 2.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#general-expectations), if you have any questions on this feel free to reach out!`,
        },
        {
          name: 'Headers as body text',
          resultingMessage: `## Description Accessibility
In accordance with section 2.2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules) we request that \`# header\`s not be used as body text. Headers are interpreted differently by screen-readers and thus should generally only be used for things like separating sections of your Description. If you would like to emphasize a particular sentence or paragraph, instead consider using \`**bold**\` text using the **B** button above the text editor.`,
        },
        {
          name: 'Image-only',
          resultingMessage: `## Image Descriptions
In accordance with section 2.2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules) we ask that you provide a text alternative to your current Description. It is important that your Description contains enough detail about your project that a user can have a full understanding of it from text alone. A text-based transcription allows for those using screen readers, and users with slow internet connections unable to load images to be able to access the contents of your Description. This also acts as a backup in case the image in your Description ever goes offline for some reason.
We appreciate how much effort you put into your Description, but accessibility is important to us at Modrinth, if you would like you could put the transcription of your Description entirely in a \`details\` tag, so as to not spoil the visuals of your Description.`,
        },
        {
          name: 'Non-standard text',
          resultingMessage: `## Description Accessibility
Per section 2 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#clear-and-honest-function) your description must be plainly readable and accessible. Using non-standard text characters like Zalgo or "fancy text" in place of text anywhere in your project, including the Description, Summary, or Title can make your project pages inaccessible. This is important for users who rely on Screen Readers and for search engines in order to provide relevant results to users. Please remove any instances of this type of text.`,
        },
      ],
    },
    {
      id: 'links',
      question: `Are the project's links accessible and not misleading?`,
      shown:
        props.project.issues_url ||
        props.project.source_url ||
        props.project.wiki_url ||
        props.project.discord_url ||
        props.project.donation_urls.length > 0,
      rules: [
        `All links must be accessible.`,
        `All links must correspond correctly with a label (ex: Discord link should not go to a YouTube channel)`,
      ],
      options: [
        {
          name: 'Links are misused',
          resultingMessage: `## Misuse of External Resources
Per section 5.4 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) all links must lead to correctly labeled publicly available resources that are directly related to your project.`,
        },
        {
          name: 'Not accessible (source)',
          resultingMessage: `## Unreachable Links
Per section 5.4 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) all links must lead to correctly labeled publicly available resources that are directly related to your project.
Currently, your Source link directs to a Page Not Found error, likely because your repository is private, make sure to make your repository public before resubmitting your project!`,
        },
        {
          name: 'Not accessible (other)',
          resultingMessage: `## Unreachable Links
Per section 5.4 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) all links must lead to correctly labeled publicly available resources that are directly related to your project.
Currently, your %LINK% link is inaccessible!`,
          fillers: [
            {
              id: 'LINK',
              question: 'Please specify the link type that is inaccessible.',
            },
          ],
        },
      ],
    },
    {
      id: 'categories',
      question: `Are the project's tags/categories accurate?`,
      shown: props.project.categories.length > 0 || props.project.additional_categories.length > 0,
      options: [
        {
          name: 'Inaccurate',
          resultingMessage: `## Misuse of Tags
Per section 5.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous), it is important that the metadata of your projects is accurate. Including that selected tags honestly represent your project.`,
        },
      ],
    },
    {
      id: 'side-types',
      question: `Is the project's environment information accurate?`,
      shown: ['mod', 'modpack'].includes(props.project.project_type),
      options: [
        {
          name: 'Inaccurate (modpack)',
          resultingMessage: `## Incorrect Environment Information
Per section 5.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous), it is important that the metadata of your projects is accurate, including whether the project runs on the client or server side.
For a brief rundown of how this works:
Some modpacks can be client-side, usually aimed at providing utility and optimization while allowing the player to join an unmodded server, for instance, [Fabulously Optimized](https://modrinth.com/modpack/fabulously-optimized).
Most other modpacks that change how the game is played are going to be required on both the client and server, like the modpack [Dying Light](https://modrinth.com/modpack/dying-light).
When in doubt, test for yourself or check the requirements of the mods in your pack.`,
        },
        {
          name: 'Inaccurate (mod)',
          resultingMessage: `## Environment Information
Per section 5.1 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous), it is important that the metadata of your projects is accurate, including whether the project runs on the client or server side.
For a brief rundown of how this works:
**Client side** refers to a mod that is only required by the client, like [Sodium](https://modrinth.com/mod/sodium).
**Server side** mods change the behavior of the server without the client needing the mod, like Datapacks, recipes, or server-side behaviors, like [Falling Tree](https://modrinth.com/mod/fallingtree).
A mod that adds features, entities, or new blocks and items, generally will be required on **both** the server and the client, for example [Cobblemon](https://modrinth.com/mod/cobblemon).`,
        },
      ],
    },
    {
      id: 'gallery',
      navigate: `/${props.project.project_type}/${props.project.slug}/gallery`,
      question: `Are the project's gallery images relevant?`,
      shown: props.project.gallery.length > 0,
      options: [
        {
          name: 'Not relevant',
          resultingMessage: `## Unrelated Gallery Images
Per section 5.5 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) any images in your project's Gallery must be relevant to the project and also include a Title.`,
        },
      ],
    },
    {
      id: 'versions',
      navigate: `/${props.project.project_type}/${props.project.slug}/versions`,
      question: `Are these project's files correct?`,
      shown: !['modpack'].includes(props.project.project_type),
      rules: [
        'A multi-loader project should not use additional files for more loaders',
        'Modpacks must be uploaded as MRPACK files. Be sure to check the project type is modpack (if not their file is malformed)',
      ],
      options: [
        {
          name: 'Incorrect additional files',
          resultingMessage: `## Incorrect Use of Additional Files
It looks like you've uploaded multiple \`mod.jar\` files to one Version as Additional Files. Per section 5.7 of [Modrinth's Content Rules](https://modrinth.com/legal/rules#miscellaneous) each Version of your project must include only one \`mod.jar\` that corresponds to its respective Minecraft and loader versions. This allows users to easily find and download the file they need for the version they're on with ease. The Additional Files feature can be used for things like a \`Sources.jar\`.
Please upload each version of your mod separately, thank you.`,
        },
        {
          name: 'Invalid file type (modpacks)',
          resultingMessage: `## Modpacks on Modrinth
It looks like you've uploaded your Modpack as a \`.zip\`, unfortunately, this is invalid and is why your project type is "Mod". I recommend taking a look at our support page about [Modrinth Modpacks](https://support.modrinth.com/en/articles/8802250-modpacks-on-modrinth), and once you're ready feel free to resubmit your project as a \`.mrpack\`. Don't forget to delete the old files from your Versions!`,
        },
        {
          name: 'Invalid file type (resourcepacks)',
          resultingMessage: `## Resource Packs on Modrinth
It looks like you've selected loaders for your Resource Pack that are causing it to be marked as a different project type. Resource Packs must only be uploaded with the "Resource Pack" loader selected. Please re-upload all versions of your resource pack and make sure to only select "Resource Pack" as the loader.`,
        },
      ],
    },
    {
      id: 'copyright',
      question: `Does the author have proper permissions to post this project?`,
      shown: true,
      rules: [
        `Perform a quick search to make sure someone is not reuploading content.`,
        `If the authors don't align or it seems like a re-post, reject and ask the user for proof they have permission.`,
      ],
      options: [
        {
          name: 'Re-upload',
          resultingMessage: `## Reuploads are forbidden
This project appears to contain content from %ORIGINAL_PROJECT% by %ORIGINAL_AUTHOR%.
Per section 4 of [Modrinth's Content Rules](https://modrinth.com/legal/rules) this is strictly forbidden.
If you believe this is an error, or you can verify you are the creator and rightful owner of this content please let us know. Otherwise, we ask that you **do not resubmit this project**.`,
          fillers: [
            {
              id: 'ORIGINAL_PROJECT',
              question: 'What is the title of the original project?',
              required: true,
            },
            {
              id: 'ORIGINAL_AUTHOR',
              question: 'What is the author of the original project?',
              required: true,
            },
          ],
        },
      ],
    },
    {
      id: 'rule-following',
      question: `Does this project follow our content rules?`,
      navigate: `/${props.project.project_type}/${props.project.slug}`,
      shown: true,
      rules: [
        'Should not be a cheat/hack (without a server-side opt-out)',
        'Should not contain sexually explicit / inappropriate content',
        'Should not be excessively profane',
        'Should not promote any illegal activity (including illicit drugs + substances)',
        'Anything else infringing of our content rules (see 1.1-12, 3.1-3)',
      ],
      options: [
        {
          name: 'No',
          resultingMessage: `%MESSAGE%`,
          fillers: [
            {
              id: 'MESSAGE',
              question: 'Please explain to the user how it infringes on our content rules.',
              large: true,
            },
          ],
        },
      ],
    },
    {
      id: 'modpack-permissions',
      question: 'Modpack permissions',
      shown: ['modpack'].includes(props.project.project_type),
      options: [],
    },
    {
      id: 'private-server',
      question: `Is this pack for a private server?`,
      shown: ['modpack'].includes(props.project.project_type),
      rules: [
        'Select this if you are withholding this pack since it is for a private server (for circumstances you would normally reject for).',
      ],
      options: [
        {
          name: 'Private server (withhold)',
          resultingMessage: `## Private Server
Under normal circumstances, your project would be rejected due to the issues listed above. However, since your project is intended for a specific server and not for general use, these requirements will be waived and your project will be withheld. This means it will be unlisted and accessible only through a direct link, without appearing in public search results. If you're fine with this, no further action is needed. Otherwise, feel free to resubmit once all issues have been addressed. `,
        },
      ],
    },
  ].filter((x) => x.shown)
)

const currentStepIndex = ref(0)
const selectedOptions = ref({})

function toggleOption(stepId, option) {
  if (!selectedOptions.value[stepId]) {
    selectedOptions.value[stepId] = []
  }

  const index = selectedOptions.value[stepId].findIndex((x) => x.name === option.name)
  if (index === -1) {
    selectedOptions.value[stepId].push(option)
  } else {
    selectedOptions.value[stepId].splice(index, 1)
  }

  const instance = getCurrentInstance()
  instance?.proxy?.$forceUpdate()
}

function previousPage() {
  currentStepIndex.value -= 1
  generatedMessage.value = false

  if (steps.value[currentStepIndex.value].navigate) {
    navigateTo(steps.value[currentStepIndex.value].navigate)
  }
}

async function nextPage() {
  currentStepIndex.value += 1

  if (steps.value[currentStepIndex.value].navigate) {
    navigateTo(steps.value[currentStepIndex.value].navigate)
  }

  if (steps.value[currentStepIndex.value].id === 'modpack-permissions') {
    await initializeModPackData()
  }
}

async function initializeModPackData() {
  startLoading()
  try {
    const raw = await useBaseFetch(`moderation/project/${props.project.id}`, { internal: true })
    const projects = []

    for (const [hash, fileName] of Object.entries(raw.unknown_files)) {
      projects.push({
        type: 'unknown',
        hash,
        file_name: fileName,
        status: null,
        approved: null,
      })
    }

    for (const [hash, file] of Object.entries(raw.flame_files)) {
      projects.push({
        type: 'flame',
        hash,
        file_name: file.file_name,
        status: null,
        title: file.title,
        id: file.id,
        url: file.url,
        approved: null,
      })
    }

    for (const [hash, file] of Object.entries(raw.identified)) {
      if (file.status !== 'yes' && file.status !== 'with-attribution-and-source') {
        projects.push({
          type: 'identified',
          hash,
          file_name: file.file_name,
          status: file.status,
          approved: null,
        })
      }
    }

    modPackData.value = projects
  } catch (err) {
    const app = useNuxtApp()
    app.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
  }
  stopLoading()
}

const modPackData = ref(null)
const modPackIndex = ref(0)

const fileApprovalTypes = ref([
  {
    id: 'yes',
    name: 'Yes',
  },
  {
    id: 'with-attribution-and-source',
    name: 'With attribution and source',
  },
  {
    id: 'with-attribution',
    name: 'With attribution',
  },
  {
    id: 'no',
    name: 'No',
  },
  {
    id: 'permanent-no',
    name: 'Permanent no',
  },
  {
    id: 'unidentified',
    name: 'Unidentified',
  },
])
const filePermissionTypes = ref([
  {
    id: true,
    name: 'Yes',
  },
  {
    id: false,
    name: 'No',
  },
])

const message = ref('')
const generatedMessage = ref(false)
const loadingMessage = ref(false)
async function generateMessage() {
  message.value = ''
  loadingMessage.value = true
  function printMods(mods, msg) {
    if (mods.length === 0) {
      return
    }

    message.value += msg
    message.value += '\n\n'

    for (const mod of mods) {
      message.value += `- ${mod}\n`
    }
  }

  if (modPackData.value && modPackData.value.length > 0) {
    const updateProjects = {}

    const attributeMods = []
    const noMods = []
    const permanentNoMods = []
    const unidentifiedMods = []

    for (const project of modPackData.value) {
      if (project.type === 'unknown') {
        updateProjects[project.hash] = {
          type: 'unknown',
          status: project.status,
          proof: project.proof,
          title: project.title,
          link: project.url,
        }
      }

      if (project.type === 'flame') {
        updateProjects[project.hash] = {
          type: 'flame',
          status: project.status,
          id: project.id,
          link: project.url,
          title: project.title,
        }
      }

      if (project.status === 'with-attribution' && !project.approved) {
        attributeMods.push(project.file_name)
      } else if (project.status === 'unidentified' && !project.approved) {
        unidentifiedMods.push(project.file_name)
      } else if (project.status === 'no' && !project.approved) {
        noMods.push(project.file_name)
      } else if (project.status === 'permanent-no') {
        permanentNoMods.push(project.file_name)
      }
    }

    if (updateProjects) {
      try {
        await useBaseFetch(`moderation/project`, {
          internal: true,
          method: 'POST',
          body: updateProjects,
        })
      } catch (err) {
        const app = useNuxtApp()
        app.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.data ? err.data.description : err,
          type: 'error',
        })
      }
    }

    if (
      attributeMods.length > 0 ||
      noMods.length > 0 ||
      permanentNoMods.length > 0 ||
      unidentifiedMods.length > 0
    ) {
      message.value += '## Copyrighted Content \n'

      printMods(
        attributeMods,
        "The following content has attribution requirements, meaning that you must link back to the page where you originally found this content in your modpack description or version changelog (e.g. linking a mod's CurseForge page if you got it from CurseForge):"
      )
      printMods(
        noMods,
        'The following content is not allowed in Modrinth modpacks due to licensing restrictions. Please contact the author(s) directly for permission or remove the content from your modpack:'
      )
      printMods(
        permanentNoMods,
        "The following content is not allowed in Modrinth modpacks, regardless of permission obtained. This may be because it breaks Modrinth's content rules or because the authors, upon being contacted for permission, have declined. Please remove the content from your modpack:"
      )
      printMods(
        unidentifiedMods,
        'The following content could not be identified. Please provide proof of its origin along with proof that you have permission to include it:'
      )

      message.value += '\n\n'
    }
  }

  for (const options of Object.values(selectedOptions.value)) {
    for (const option of options) {
      let addonMessage = option.resultingMessage

      if (option.fillers && option.fillers.length > 0) {
        for (const filler of option.fillers) {
          addonMessage = addonMessage.replace(new RegExp(`%${filler.id}%`, 'g'), filler.value ?? '')
        }
      }

      message.value += addonMessage
      message.value += '\n\n'
    }
  }
  generatedMessage.value = true
  loadingMessage.value = false
  currentStepIndex.value += 1
  await navigateTo(`/${props.project.project_type}/${props.project.slug}/moderation`)
}

const done = ref(false)
async function sendMessage(status) {
  startLoading()
  try {
    await useBaseFetch(`project/${props.project.id}`, {
      method: 'PATCH',
      body: {
        status,
      },
    })

    if (message.value) {
      await useBaseFetch(`thread/${props.project.thread_id}`, {
        method: 'POST',
        body: {
          body: {
            type: 'text',
            body: message.value,
          },
        },
      })
    }

    await props.resetProject()
    done.value = true
  } catch (err) {
    const app = useNuxtApp()
    app.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data ? err.data.description : err,
      type: 'error',
    })
  }
  stopLoading()
}

const router = useNativeRouter()

async function goToNextProject() {
  const project = props.futureProjects[0]

  if (!project) {
    await navigateTo('/moderation/review')
  }

  await router.push({
    name: 'type-id',
    params: {
      type: 'project',
      id: project,
    },
    state: {
      showChecklist: true,
      projects: props.futureProjects.slice(1),
    },
  })
}
</script>

<style scoped lang="scss">
.moderation-checklist {
  position: sticky;
  bottom: 0;
  left: 100vw;
  z-index: 100;
  border: 1px solid var(--color-bg-inverted);
  width: 600px;

  .skip-btn {
    margin-right: auto;
  }

  .next-project {
    margin-left: auto;
  }

  .modpack-buttons {
    margin-top: 1rem;
  }

  .option-selected {
    color: var(--color-contrast);
    background-color: var(--color-brand-highlight);
    box-shadow: inset 0 0 0 transparent, 0 0 0 2px var(--color-brand);
  }
}
</style>
