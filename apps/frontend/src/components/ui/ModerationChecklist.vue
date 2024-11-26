<template>
  <div class="card moderation-checklist">
    <h1>审核清单</h1>
    <div v-if="done">
      <p>您已完成此项目的审核！还有 {{ futureProjects.length }} 个项目。</p>
    </div>
    <div v-else-if="generatedMessage">
      <p>
        在此输入您的审核消息。请记得检查审核标签，以回答作者可能提出的任何问题！
      </p>
      <div class="markdown-editor-spacing">
        <MarkdownEditor v-model="message" :placeholder="'输入审核消息'"/>
      </div>
    </div>
    <div v-else-if="steps[currentStepIndex].id === 'modpack-permissions'">
      <h2 v-if="modPackData">
        模组包权限
        <template v-if="modPackIndex + 1 <= modPackData.length">
          ({{ modPackIndex + 1 }} / {{ modPackData.length }})
        </template>
      </h2>
      <div v-if="!modPackData">加载数据中...</div>
      <div v-else-if="modPackData.length === 0">
        <p>所有权限已获得。您可以跳过此步骤！</p>
      </div>
      <div v-else-if="!modPackData[modPackIndex]">
        <p>所有权限检查已完成！</p>
        <div class="input-group modpack-buttons">
          <button class="btn" @click="modPackIndex -= 1">
            <LeftArrowIcon aria-hidden="true"/>
            上一步
          </button>
        </div>
      </div>
      <div v-else>
        <div v-if="modPackData[modPackIndex].type === 'unknown'">
          <p>{{ modPackData[modPackIndex].file_name }} 的批准类型是什么？</p>
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
              <span class="label__title">证明</span>
            </label>
            <input
                id="proof"
                v-model="modPackData[modPackIndex].proof"
                type="text"
                autocomplete="off"
                placeholder="输入状态证明..."
            />
            <label for="link">
              <span class="label__title">链接</span>
            </label>
            <input
                id="link"
                v-model="modPackData[modPackIndex].url"
                type="text"
                autocomplete="off"
                placeholder="输入项目链接..."
            />
            <label for="title">
              <span class="label__title">标题</span>
            </label>
            <input
                id="title"
                v-model="modPackData[modPackIndex].title"
                type="text"
                autocomplete="off"
                placeholder="输入项目标题..."
            />
          </template>
        </div>
        <div v-else-if="modPackData[modPackIndex].type === 'flame'">
          <p>
            {{ modPackData[modPackIndex].title }} 的批准类型是什么 (<a
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
            该项目是否提供了 <strong>{{ modPackData[modPackIndex].file_name }}</strong> 的识别和权限？
          </p>
          <p v-else-if="modPackData[modPackIndex].status === 'with-attribution'">
            该项目是否为 <strong>{{ modPackData[modPackIndex].file_name }}</strong> 提供了归属？
          </p>
          <p v-else>
            该项目是否提供了 <strong>{{ modPackData[modPackIndex].file_name }}</strong> 的权限证明？
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
            <LeftArrowIcon aria-hidden="true"/>
            上一步
          </button>
          <button
              class="btn btn-blue"
              :disabled="!modPackData[modPackIndex].status"
              @click="modPackIndex += 1"
          >
            <RightArrowIcon aria-hidden="true"/>
            下一个项目
          </button>
        </div>
      </div>
    </div>
    <div v-else>
      <h2>{{ steps[currentStepIndex].question }}</h2>
      <template v-if="steps[currentStepIndex].rules && steps[currentStepIndex].rules.length > 0">
        <strong>规则指导：</strong>
        <ul>
          <li v-for="(rule, index) in steps[currentStepIndex].rules" :key="index">
            {{ rule }}
          </li>
        </ul>
      </template>
      <template
          v-if="steps[currentStepIndex].examples && steps[currentStepIndex].examples.length > 0"
      >
        <strong>拒绝示例：</strong>
        <ul>
          <li v-for="(example, index) in steps[currentStepIndex].examples" :key="index">
            {{ example }}
          </li>
        </ul>
      </template>
      <template
          v-if="steps[currentStepIndex].exceptions && steps[currentStepIndex].exceptions.length > 0"
      >
        <strong>例外情况：</strong>
        <ul>
          <li v-for="(exception, index) in steps[currentStepIndex].exceptions" :key="index">
            {{ exception }}
          </li>
        </ul>
      </template>
      <p v-if="steps[currentStepIndex].id === 'title'">
        <strong>标题：</strong> {{ project.title }}
      </p>
      <p v-if="steps[currentStepIndex].id === 'slug'"><strong>Slug：</strong> {{ project.slug }}</p>
      <p v-if="steps[currentStepIndex].id === 'summary'">
        <strong>摘要：</strong> {{ project.description }}
      </p>
      <p v-if="steps[currentStepIndex].id === 'links'">
        <template v-if="project.issues_url">
          <strong>问题： </strong>
          <a class="text-link" :href="project.issues_url">{{ project.issues_url }}</a> <br/>
        </template>
        <template v-if="project.source_url">
          <strong>源代码： </strong>
          <a class="text-link" :href="project.source_url">{{ project.source_url }}</a> <br/>
        </template>
        <template v-if="project.wiki_url">
          <strong>Wiki： </strong>
          <a class="text-link" :href="project.wiki_url">{{ project.wiki_url }}</a> <br/>
        </template>
        <template v-if="project.discord_url">
          <strong>Discord： </strong>
          <a class="text-link" :href="project.discord_url">{{ project.discord_url }}</a>
          <br/>
        </template>
        <template v-for="(donation, index) in project.donation_urls" :key="index">
          <strong>{{ donation.platform }}： </strong>
          <a class="text-link" :href="donation.url">{{ donation.url }}</a>
          <br/>
        </template>
      </p>
      <p v-if="steps[currentStepIndex].id === 'categories'">
        <strong>类别：</strong>
        <Categories
            :categories="project.categories.concat(project.additional_categories)"
            :type="project.actualProjectType"
            class="categories"
        />
      </p>
      <p v-if="steps[currentStepIndex].id === 'side-types'">
        <strong>客户端：</strong> {{ project.client_side }} <br/>
        <strong>服务器端：</strong> {{ project.server_side }}
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
            (x) => x.fillers && x.fillers.length > 0,
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
              <MarkdownEditor v-model="filler.value" :placeholder="'输入审核消息'"/>
            </div>
            <input v-else :id="filler.id" v-model="filler.value" type="text" autocomplete="off"/>
          </div>
        </div>
      </div>
    </div>
    <div class="input-group modpack-buttons">
      <button v-if="!done" class="btn skip-btn" aria-label="跳过" @click="goToNextProject">
        <ExitIcon aria-hidden="true"/>
        <template v-if="futureProjects.length > 0">跳过</template>
        <template v-else>退出</template>
      </button>
      <button v-if="currentStepIndex > 0" class="btn" @click="previousPage() && !done">
        <LeftArrowIcon aria-hidden="true"/>
        上一步
      </button>
      <button
          v-if="currentStepIndex < steps.length - 1 && !done"
          class="btn btn-primary"
          @click="nextPage()"
      >
        <RightArrowIcon aria-hidden="true"/>
        下一步
      </button>
      <button
          v-else-if="!generatedMessage"
          class="btn btn-primary"
          :disabled="loadingMessage"
          @click="generateMessage"
      >
        <UpdatedIcon aria-hidden="true"/>
        生成消息
      </button>
      <template v-if="generatedMessage && !done">
        <button class="btn btn-green" @click="sendMessage(project.requested_status ?? 'approved')">
          <CheckIcon aria-hidden="true"/>
          批准
        </button>
        <div class="joined-buttons">
          <button class="btn btn-danger" @click="sendMessage('rejected')">
            <CrossIcon aria-hidden="true"/>
            拒绝
          </button>
          <OverflowMenu
              class="btn btn-danger btn-dropdown-animation icon-only"
              :options="[
              {
                id: 'withhold',
                color: 'danger',
                action: () => sendMessage('withheld'),
                hoverFilled: true,
              },
            ]"
          >
            <DropdownIcon style="rotate: 180deg"/>
            <template #withhold>
              <EyeOffIcon aria-hidden="true"/>
              保留
            </template>
          </OverflowMenu>
        </div>
      </template>
      <button v-if="done" class="btn btn-primary next-project" @click="goToNextProject">
        下一个项目
      </button>
    </div>
  </div>
</template>
<script setup>
import {
  LeftArrowIcon,
  RightArrowIcon,
  UpdatedIcon,
  CheckIcon,
  DropdownIcon,
  XIcon as CrossIcon,
  EyeOffIcon,
  ExitIcon,
} from "@modrinth/assets";
import {MarkdownEditor, OverflowMenu} from "@modrinth/ui";
import Categories from "~/components/ui/search/Categories.vue";

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
    default: () => {
    },
  },
});

const steps = computed(() =>
    [
      {
        id: "title",
        question: "标题是否没有无用信息？",
        shown: true,
        rules: [
          "没有不必要的数据（模组加载器、游戏版本等）",
          "没有表情符号/无用的文本装饰",
        ],
        examples: [
          "✅ NoobMod [1.8+] • 在你的世界中杀死所有菜鸟！",
          "[FABRIC] 我的优化包",
          "[1.17-1.20.4] LagFixer ⚡️ 最佳性能解决方案！ ⭕ 优化良好 ✅ 支持 Folia！(BETA)",
        ],
        exceptions: [
          "如果此项目是另一个模组的移植版本，则允许加载器和/或游戏版本。（例如：Gravestones for 1.20）",
          "如果他们选择将项目分为 Forge 和 Fabric 变体（不推荐），则允许加载器。",
        ],
        options: [
          {
            name: "包含无用信息",
            resultingMessage: `## 标题误用
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.2 节，我们要求您将标题限制为项目名称。其他信息，如主题、标签、支持的版本或加载器等，应保存在摘要或描述中。更改项目标题时，请记住还要确保项目 slug（URL）匹配并准确代表您的项目。`,
          },
        ],
      },
      {
        id: "slug",
        question: "slug 是否准确且合适？",
        shown: true,
        rules: ["与标题匹配/不误导（首字母缩略词可以）"],
        options: [
          {
            name: "误用",
            resultingMessage: `## Slug 误用
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.2 节，您的项目 slug（URL）必须准确代表您的项目。`,
          },
        ],
      },
      {
        id: "summary",
        question: `项目的摘要是否足够？`,
        shown: true,
        rules: [
          "摘要应提供项目的简要概述，以告知和吸引用户。",
          `不应与项目标题完全相同`,
          "不应包含任何 markdown 格式。",
        ],
        options: [
          {
            name: "不足",
            resultingMessage: `## 摘要不足
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.3 节，您的项目摘要应提供项目的简要概述，以告知和吸引用户。
这是大多数人除了 Logo 之外看到的关于您的模组的第一件事，因此它必须准确、合理详细且令人兴奋。`,
          },
          {
            name: "重复标题",
            resultingMessage: `## 摘要不足
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.3 节，您的摘要不能与项目标题相同。您的项目摘要应提供项目的简要概述，以告知和吸引用户。
这是大多数人除了 Logo 之外看到的关于您的模组的第一件事，因此它必须准确、合理详细且令人兴奋。`,
          },
          {
            name: "格式化",
            resultingMessage: `## 摘要不足
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.3 节，您的摘要不能包含任何额外的格式，如列表或链接。您的项目摘要应提供项目的简要概述，以告知和吸引用户。
这是大多数人除了 Logo 之外看到的关于您的模组的第一件事，因此它必须准确、合理详细且令人兴奋。`,
          },
        ],
      },
      {
        id: "description",
        question: `项目的描述是否足够？`,
        navigate: `/${props.project.project_type}/${props.project.slug}`,
        shown: true,
        rules: [
          "应回答项目具体做了什么或添加了什么",
          "应回答为什么有人会想下载这个项目",
          "应指出用户在下载前必须知道的任何其他关键信息",
          "应易于访问（没有花哨的字符/非标准文本，没有仅图像描述，必须有英文部分等）",
        ],
        options: [
          {
            name: "不足",
            resultingMessage: `## 描述不足
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#general-expectations) 第 2.1 节，您的项目描述应清楚地告知读者项目的内容、目的和吸引力。
目前，看起来有一些缺失的细节。
%EXPLAINER%`,
            fillers: [
              {
                id: "EXPLAINER",
                question: "请详细说明作者如何改进他们的描述。",
                large: true,
              },
            ],
          },
          {
            name: "不足（默认包）",
            resultingMessage: `## 描述不足
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#general-expectations) 第 2.1 节，您的项目描述应清楚地告知读者项目的内容、目的和吸引力。
目前，看起来有一些缺失的细节。
您的模组包添加了什么？它有哪些功能？为什么用户会想下载它？请具体说明！
请参阅 [Simply Optimized](https://bbsmc.net/modpack/sop) 或 [Aged](https://bbsmc.net/modpack/aged) 的描述示例，了解良好描述的样子。
`,
          },
          {
            name: "不足（默认项目）",
            resultingMessage: `## 描述不足
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#general-expectations) 第 2.1 节，您的项目描述应清楚地告知读者项目的内容、目的和吸引力。
目前，看起来有一些缺失的细节。
您的项目添加了什么？它有哪些功能？为什么用户会想下载它？请具体说明！
请参阅 [Sodium](https://bbsmc.net/mod/sodium) 或 [LambDynamicLights](https://bbsmc.net/mod/lambdynamiclights) 的描述示例，了解良好描述的样子。
`,
          },
          {
            name: "非英文",
            resultingMessage: `## 没有英文描述
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#accessibility) 第 2.2 节，项目的摘要和描述必须是英文的，除非专门用于非英文使用，如翻译。您可以包含非英文描述，但我们要求您也在描述页面添加英文翻译，如果您想使用在线翻译器，我们推荐 [DeepL](https://www.deepl.com/translator)。`,
          },
          {
            name: "未完成",
            resultingMessage: `## 描述未完成
看起来您的项目描述仍在进行中，因为 %REASON%。请记住只有在准备好时提交，因为您的项目必须符合 [BBSMC 内容规则](https://bbsmc.net/legal/rules#general-expectations) 第 2.1 节的要求，如果您对此有任何疑问，请随时联系我们！`,
          },
          {
            name: "标题作为正文",
            resultingMessage: `## 描述可访问性
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules) 第 2.2 节，我们要求不要将 \`# 标题\` 用作正文。标题在屏幕阅读器中的解释不同，因此通常只应用于分隔描述的部分。如果您想强调特定的句子或段落，请考虑使用 \`**粗体**\` 文本，使用文本编辑器上方的 **B** 按钮。`,
          },
          {
            name: "仅图像",
            resultingMessage: `## 图像描述
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules) 第 2.2 节，我们要求您为当前描述提供文本替代。您的描述应包含足够的项目详细信息，以便用户仅通过文本即可全面了解它。文本转录允许使用屏幕阅读器的用户和无法加载图像的慢速互联网用户访问描述内容。这也作为备份，以防描述中的图像因某种原因离线。
我们感谢您在描述中付出的努力，但可访问性对我们来说很重要，如果您愿意，您可以将描述的转录完全放在 \`details\` 标签中，以免破坏描述的视觉效果。`,
          },
          {
            name: "非标准文本",
            resultingMessage: `## 描述可访问性
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#clear-and-honest-function) 第 2 节，您的描述必须清晰可读且易于访问。使用非标准文本字符，如 Zalgo 或“花哨文本”代替文本，可能会使您的项目页面无法访问。这对依赖屏幕阅读器的用户和搜索引擎提供相关结果非常重要。请删除此类文本的任何实例。`,
          },
        ],
      },
      {
        id: "links",
        question: `项目的链接是否可访问且不误导？`,
        shown:
            props.project.issues_url ||
            props.project.source_url ||
            props.project.wiki_url ||
            props.project.discord_url ||
            props.project.donation_urls.length > 0,
        rules: [
          `所有链接必须可访问。`,
          `所有链接必须正确对应标签（例如：Discord 链接不应指向 YouTube 频道）`,
        ],
        options: [
          {
            name: "链接被误用",
            resultingMessage: `## 外部资源误用
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.4 节，所有链接必须指向正确标记的公开可用资源，并且与您的项目直接相关。`,
          },
          {
            name: "不可访问（源）",
            resultingMessage: `## 链接不可访问
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.4 节，所有链接必须指向正确标记的公开可用资源，并且与您的项目直接相关。
目前，您的源链接指向“页面未找到”错误，可能是因为您的存储库是私有的，请确保在重新提交项目之前将存储库设为公开！`,
          },
          {
            name: "不可访问（其他）",
            resultingMessage: `## 链接不可访问
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.4 节，所有链接必须指向正确标记的公开可用资源，并且与您的项目直接相关。
目前，您的 %LINK% 链接不可访问！`,
            fillers: [
              {
                id: "LINK",
                question: "请指定不可访问的链接类型。",
              },
            ],
          },
        ],
      },
      {
        id: "categories",
        question: `项目的标签/类别是否准确？`,
        shown: props.project.categories.length > 0 || props.project.additional_categories.length > 0,
        options: [
          {
            name: "不准确",
            resultingMessage: `## 标签误用
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.1 节，项目的元数据必须准确。包括所选标签诚实地代表您的项目。`,
          },
        ],
      },
      {
        id: "side-types",
        question: `项目的环境信息是否准确？`,
        shown: ["mod", "modpack"].includes(props.project.project_type),
        options: [
          {
            name: "不准确（模组包）",
            resultingMessage: `## 环境信息不正确
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.1 节，项目的元数据必须准确，包括项目在客户端或服务器端运行。
简要介绍如下：
一些模组包可以是客户端的，通常旨在提供实用程序和优化，同时允许玩家加入未修改的服务器，例如 [Fabulously Optimized](https://bbsmc.net/modpack/fabulously-optimized)。
大多数其他改变游戏玩法的模组包将在客户端和服务器上都需要，例如模组包 [Dying Light](https://bbsmc.net/modpack/dying-light)。
如有疑问，请自行测试或检查包中模组的要求。`,
          },
          {
            name: "不准确（模组）",
            resultingMessage: `## 环境信息
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.1 节，项目的元数据必须准确，包括项目在客户端或服务器端运行。
简要介绍如下：
**客户端** 指仅客户端需要的模组，例如 [Sodium](https://bbsmc.net/mod/sodium)。
**服务器端** 模组更改服务器的行为，而无需客户端需要模组，例如数据包、配方或服务器端行为，例如 [Falling Tree](https://bbsmc.net/mod/fallingtree)。
添加功能、实体或新方块和物品的模组，通常在 **客户端和服务器** 上都需要，例如 [Cobblemon](https://bbsmc.net/mod/cobblemon)。`,
          },
        ],
      },
      {
        id: "gallery",
        navigate: `/${props.project.project_type}/${props.project.slug}/gallery`,
        question: `项目的画廊图片是否相关？`,
        shown: props.project.gallery.length > 0,
        options: [
          {
            name: "不相关",
            resultingMessage: `## 无关的画廊图片
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.5 节，项目画廊中的任何图片必须与项目相关，并且包含标题。`,
          },
        ],
      },
      {
        id: "versions",
        navigate: `/${props.project.project_type}/${props.project.slug}/versions`,
        question: `这些项目的文件是否正确？`,
        shown: !["modpack"].includes(props.project.project_type),
        rules: [
          "多加载器项目不应使用额外文件来加载更多加载器",
          "模组包必须作为 MRPACK 文件上传。请确保项目类型为模组包（如果不是，则文件格式错误）",
        ],
        options: [
          {
            name: "额外文件不正确",
            resultingMessage: `## 额外文件使用不正确
看起来您已将多个 \`mod.jar\` 文件作为额外文件上传到一个版本。根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules#miscellaneous) 第 5.7 节，每个版本的项目必须仅包含一个 \`mod.jar\`，对应其各自的 Minecraft 和加载器版本。这使用户可以轻松找到并下载他们所需版本的文件。额外文件功能可用于诸如 \`Sources.jar\` 之类的内容。
请分别上传每个版本的模组，谢谢。`,
          },
          {
            name: "文件类型无效（模组包）",
            resultingMessage: `## Modrinth 上的模组包
看起来您已将模组包上传为 \`.zip\`，不幸的是，这是无效的，这就是为什么您的项目类型是“模组”。我建议查看我们的支持页面 [Modrinth Modpacks](https://support.modrinth.com/en/articles/8802250-modpacks-on-modrinth)，一旦准备好，请随时重新提交项目为 \`.mrpack\`。不要忘记从版本中删除旧文件！`,
          },
          {
            name: "文件类型无效（资源包）",
            resultingMessage: `## Modrinth 上的资源包
看起来您为资源包选择的加载器导致其被标记为不同的项目类型。资源包必须仅使用“资源包”加载器上传。请重新上传所有版本的资源包，并确保仅选择“资源包”作为加载器。`,
          },
        ],
      },
      {
        id: "copyright",
        question: `作者是否有适当的权限发布此项目？`,
        shown: true,
        rules: [
          `快速搜索以确保没有人重新上传内容。`,
          `如果作者不一致或看起来像是重新发布，请拒绝并要求用户提供他们有权限的证明。`,
        ],
        options: [
          {
            name: "重新上传",
            resultingMessage: `## 禁止重新上传
该项目似乎包含 %ORIGINAL_PROJECT% 的内容，由 %ORIGINAL_AUTHOR% 创建。
根据 [BBSMC 内容规则](https://bbsmc.net/legal/rules) 第 4 节，这是严格禁止的。
如果您认为这是一个错误，或者您可以验证您是此内容的创建者和合法所有者，请告知我们。否则，我们要求您 **不要重新提交此项目**。`,
            fillers: [
              {
                id: "ORIGINAL_PROJECT",
                question: "原始项目的标题是什么？",
                required: true,
              },
              {
                id: "ORIGINAL_AUTHOR",
                question: "原始项目的作者是谁？",
                required: true,
              },
            ],
          },
        ],
      },
      {
        id: "rule-following",
        question: `此项目是否遵循我们的内容规则？`,
        navigate: `/${props.project.project_type}/${props.project.slug}`,
        shown: true,
        rules: [
          "不应是作弊/黑客（没有服务器端选择退出）",
          "不应包含性暗示/不适当的内容",
          "不应过度使用粗话",
          "不应宣传任何非法活动（包括非法药物和物质）",
          "任何其他违反我们内容规则的内容（见 1.1-12，3.1-3）",
        ],
        options: [
          {
            name: "否",
            resultingMessage: `%MESSAGE%`,
            fillers: [
              {
                id: "MESSAGE",
                question: "请向用户解释它如何违反我们的内容规则。",
                large: true,
              },
            ],
          },
        ],
      },
      {
        id: "modpack-permissions",
        question: "模组包权限",
        shown: ["modpack"].includes(props.project.project_type),
        options: [],
      },
      {
        id: "private-server",
        question: `这个包是为私人服务器准备的吗？`,
        shown: ["modpack"].includes(props.project.project_type),
        rules: [
          "如果您因为这是为私人服务器准备的包而保留它（在通常情况下您会拒绝的情况），请选择此选项。",
        ],
        options: [
          {
            name: "私人服务器（保留）",
            resultingMessage: `## 私人服务器
在正常情况下，由于上述问题，您的项目将被拒绝。然而，由于您的项目是为特定服务器准备的，而不是用于一般用途，这些要求将被免除，您的项目将被保留。这意味着它将被取消列出，并且只能通过直接链接访问，而不会出现在公共搜索结果中。如果您对此没有意见，则无需采取进一步行动。否则，请在解决所有问题后随时重新提交。`,
          },
        ],
      },
    ].filter((x) => x.shown),
);

const currentStepIndex = ref(0);
const selectedOptions = ref({});

function toggleOption(stepId, option) {
  if (!selectedOptions.value[stepId]) {
    selectedOptions.value[stepId] = [];
  }

  const index = selectedOptions.value[stepId].findIndex((x) => x.name === option.name);
  if (index === -1) {
    selectedOptions.value[stepId].push(option);
  } else {
    selectedOptions.value[stepId].splice(index, 1);
  }

  const instance = getCurrentInstance();
  instance?.proxy?.$forceUpdate();
}

function previousPage() {
  currentStepIndex.value -= 1;
  generatedMessage.value = false;

  if (steps.value[currentStepIndex.value].navigate) {
    navigateTo(steps.value[currentStepIndex.value].navigate);
  }
}

async function nextPage() {
  currentStepIndex.value += 1;

  if (steps.value[currentStepIndex.value].navigate) {
    navigateTo(steps.value[currentStepIndex.value].navigate);
  }

  if (steps.value[currentStepIndex.value].id === "modpack-permissions") {
    await initializeModPackData();
  }
}

async function initializeModPackData() {
  startLoading();
  try {
    const raw = await useBaseFetch(`moderation/project/${props.project.id}`, {internal: true});
    const projects = [];

    for (const [hash, fileName] of Object.entries(raw.unknown_files)) {
      projects.push({
        type: "unknown",
        hash,
        file_name: fileName,
        status: null,
        approved: null,
      });
    }

    for (const [hash, file] of Object.entries(raw.flame_files)) {
      projects.push({
        type: "flame",
        hash,
        file_name: file.file_name,
        status: null,
        title: file.title,
        id: file.id,
        url: file.url,
        approved: null,
      });
    }

    for (const [hash, file] of Object.entries(raw.identified)) {
      if (file.status !== "yes" && file.status !== "with-attribution-and-source") {
        projects.push({
          type: "identified",
          hash,
          file_name: file.file_name,
          status: file.status,
          approved: null,
        });
      }
    }

    modPackData.value = projects;
  } catch (err) {
    const app = useNuxtApp();
    app.$notify({
      group: "main",
      title: "发生错误",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

const modPackData = ref(null);
const modPackIndex = ref(0);

const fileApprovalTypes = ref([
  {
    id: "yes",
    name: "Yes",
  },
  {
    id: "with-attribution-and-source",
    name: "With attribution and source",
  },
  {
    id: "with-attribution",
    name: "With attribution",
  },
  {
    id: "no",
    name: "No",
  },
  {
    id: "permanent-no",
    name: "Permanent no",
  },
  {
    id: "unidentified",
    name: "Unidentified",
  },
]);
const filePermissionTypes = ref([
  {
    id: true,
    name: "Yes",
  },
  {
    id: false,
    name: "No",
  },
]);

const message = ref("");
const generatedMessage = ref(false);
const loadingMessage = ref(false);

async function generateMessage() {
  message.value = "";
  loadingMessage.value = true;

  function printMods(mods, msg) {
    if (mods.length === 0) {
      return;
    }

    message.value += msg;
    message.value += "\n\n";

    for (const mod of mods) {
      message.value += `- ${mod}\n`;
    }
  }

  if (modPackData.value && modPackData.value.length > 0) {
    const updateProjects = {};

    const attributeMods = [];
    const noMods = [];
    const permanentNoMods = [];
    const unidentifiedMods = [];

    for (const project of modPackData.value) {
      if (project.type === "unknown") {
        updateProjects[project.hash] = {
          type: "unknown",
          status: project.status,
          proof: project.proof,
          title: project.title,
          link: project.url,
        };
      }

      if (project.type === "flame") {
        updateProjects[project.hash] = {
          type: "flame",
          status: project.status,
          id: project.id,
          link: project.url,
          title: project.title,
        };
      }

      if (project.status === "with-attribution" && !project.approved) {
        attributeMods.push(project.file_name);
      } else if (project.status === "unidentified" && !project.approved) {
        unidentifiedMods.push(project.file_name);
      } else if (project.status === "no" && !project.approved) {
        noMods.push(project.file_name);
      } else if (project.status === "permanent-no") {
        permanentNoMods.push(project.file_name);
      }
    }

    if (updateProjects) {
      try {
        await useBaseFetch(`moderation/project`, {
          internal: true,
          method: "POST",
          body: updateProjects,
        });
      } catch (err) {
        const app = useNuxtApp();
        app.$notify({
          group: "main",
          title: "发生错误",
          text: err.data ? err.data.description : err,
          type: "error",
        });
      }
    }

    if (
        attributeMods.length > 0 ||
        noMods.length > 0 ||
        permanentNoMods.length > 0 ||
        unidentifiedMods.length > 0
    ) {
      message.value += "## 版权内容\n";

      printMods(
          attributeMods,
          "以下内容有归属要求，这意味着您必须在模组包描述或版本更新日志中链接回您最初找到此内容的页面（例如，如果您从 CurseForge 获取模组，请链接模组的 CurseForge 页面）：",
      );
      printMods(
          noMods,
          "由于许可限制，以下内容不允许在 Modrinth 模组包中使用。请直接联系作者获取许可或从模组包中删除内容：",
      );
      printMods(
          permanentNoMods,
          "无论是否获得许可，以下内容都不允许在 Modrinth 模组包中使用。这可能是因为它违反了 BBSMC 内容规则，或者因为在联系作者获取许可时被拒绝。请从模组包中删除内容：",
      );
      printMods(
          unidentifiedMods,
          "以下内容无法识别。请提供其来源的证明以及您有权包含它的证明：",
      );
      message.value += "\n\n";
    }
  }

  for (const options of Object.values(selectedOptions.value)) {
    for (const option of options) {
      let addonMessage = option.resultingMessage;

      if (option.fillers && option.fillers.length > 0) {
        for (const filler of option.fillers) {
          addonMessage = addonMessage.replace(
              new RegExp(`%${filler.id}%`, "g"),
              filler.value ?? "",
          );
        }
      }

      message.value += addonMessage;
      message.value += "\n\n";
    }
  }
  generatedMessage.value = true;
  loadingMessage.value = false;
  currentStepIndex.value += 1;
  await navigateTo(`/${props.project.project_type}/${props.project.slug}/moderation`);
}

const done = ref(false);

async function sendMessage(status) {
  startLoading();
  try {
    await useBaseFetch(`project/${props.project.id}`, {
      method: "PATCH",
      body: {
        status,
      },
    });

    if (message.value) {
      await useBaseFetch(`thread/${props.project.thread_id}`, {
        method: "POST",
        body: {
          body: {
            type: "text",
            body: message.value,
          },
        },
      });
    }

    await props.resetProject();
    done.value = true;
  } catch (err) {
    const app = useNuxtApp();
    app.$notify({
      group: "main",
      title: "发生错误",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

const router = useNativeRouter();

async function goToNextProject() {
  const project = props.futureProjects[0];

  if (!project) {
    await navigateTo("/moderation/review");
  }

  await router.push({
    name: "type-id",
    params: {
      type: "project",
      id: project,
    },
    state: {
      showChecklist: true,
      projects: props.futureProjects.slice(1),
    },
  });
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
    box-shadow: inset 0 0 0 transparent,
    0 0 0 2px var(--color-brand);
  }
}
</style>
