<template>
  <div class="page">
    <div class="experimental-styles-within flex flex-col gap-2">
      <RadialHeader class="top-box mb-2 flex flex-col items-center justify-center" color="orange">
        <ScaleIcon class="h-12 w-12 text-brand-orange" />
        <h1 class="m-3 gap-2 text-3xl font-extrabold">
          {{
            prefilled && itemName
              ? existingReport
                ? formatMessage(messages.alreadyReportedItem, { title: itemName })
                : formatMessage(messages.reportItem, { title: itemName })
              : formatMessage(messages.reportContent)
          }}
        </h1>
      </RadialHeader>
      <div
        v-if="prefilled && itemName && existingReport"
        class="mx-auto flex max-w-[35rem] flex-col items-center gap-4 text-center"
      >
        {{ formatMessage(messages.alreadyReportedDescription, { item: reportItem || "content" }) }}
        <div class="flex gap-2">
          <ButtonStyled v-if="itemLink">
            <nuxt-link :to="itemLink">
              <LeftArrowIcon />
              {{ formatMessage(messages.backToItem, { item: reportItem || "content" }) }}
            </nuxt-link>
          </ButtonStyled>
          <ButtonStyled color="brand">
            <nuxt-link :to="`/dashboard/report/${existingReport.id}`">
              {{ formatMessage(messages.goToReport) }} <RightArrowIcon />
            </nuxt-link>
          </ButtonStyled>
        </div>
      </div>
      <template v-else>
        <div class="mb-3 grid grid-cols-1 gap-4 px-6 md:grid-cols-2">
          <div class="flex flex-col gap-2">
            <h2 class="m-0 text-lg font-extrabold">{{ formatMessage(messages.pleaseReport) }}</h2>
            <div class="text-md flex items-center gap-2 font-semibold text-contrast">
              <CheckCircleIcon class="h-8 w-8 shrink-0 text-brand-green" />
              <div class="flex flex-col">
                <span>
                  <IntlFormatted :message-id="messages.violation">
                    <template #rules-link="{ children }">
                      <nuxt-link class="text-link" :to="`/legal/rules`">
                        <component :is="() => children" />
                      </nuxt-link>
                    </template>
                    <template #terms-link="{ children }">
                      <nuxt-link class="text-link" :to="`/legal/terms`">
                        <component :is="() => children" />
                      </nuxt-link>
                    </template>
                  </IntlFormatted>
                </span>
                <span class="text-sm font-medium text-secondary">
                  {{ formatMessage(messages.violationDescription) }}
                </span>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <h2 class="m-0 text-lg font-extrabold">{{ formatMessage(messages.formNotFor) }}</h2>
            <div class="text-md flex items-center gap-2 font-semibold text-contrast">
              <XCircleIcon class="h-8 w-8 shrink-0 text-brand-red" />

              <div class="flex flex-col">
                <span>{{ formatMessage(messages.bugReports) }}</span>
                <span v-if="itemIssueTracker" class="text-sm font-medium text-secondary">
                  <IntlFormatted :message-id="messages.bugReportsDescription">
                    <template #issues-link="{ children }">
                      <a class="text-link" :href="itemIssueTracker" target="_blank">
                        <component :is="() => children" />
                        <ExternalIcon aria-hidden="true" class="mb-1 ml-1 h-2.5 w-2.5" />
                      </a>
                    </template>
                  </IntlFormatted>
                </span>
              </div>
            </div>
            <div class="text-md flex items-center gap-2 font-semibold text-contrast">
              <XCircleIcon class="h-8 w-8 shrink-0 text-brand-red" />
              <div class="flex flex-col">
                <span>{{ formatMessage(messages.dmcaTakedown) }}</span>
                <span class="text-sm font-medium text-secondary">
                  <IntlFormatted :message-id="messages.dmcaTakedownDescription">
                    <template #policy-link="{ children }">
                      <nuxt-link class="text-link" :to="`/legal/copyright`">
                        <component :is="() => children" />
                      </nuxt-link>
                    </template>
                  </IntlFormatted>
                </span>
              </div>
            </div>
          </div>
        </div>
        <div class="flex flex-col gap-4 rounded-xl bg-bg-raised p-6">
          <template v-if="!prefilled || !currentItemValid">
            <div class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">
                {{ formatMessage(messages.whatContentType) }}
              </span>
              <RadioButtons
                v-slot="{ item }"
                v-model="reportItem"
                :items="reportItems"
                @update:model-value="
                  () => {
                    prefilled = false;
                    fetchItem();
                  }
                "
              >
                {{ capitalizeString(item) }}
              </RadioButtons>
            </div>
            <div class="flex flex-col gap-2" :class="{ hidden: !reportItem }">
              <span class="text-lg font-bold text-contrast">
                {{ formatMessage(messages.whatContentId, { item: reportItem || "content" }) }}
              </span>
              <div class="flex gap-4">
                <input
                  id="report-item-id"
                  v-model="reportItemID"
                  type="text"
                  placeholder="ex: Dc7EYhxG"
                  autocomplete="off"
                  :disabled="reportItem === ''"
                  class="w-40"
                  @blur="
                    () => {
                      prefilled = false;
                      reportItemID = reportItemID.trim();
                      fetchItem();
                    }
                  "
                />
                <div v-if="checkingId || checkedId" class="flex items-center gap-1">
                  <template v-if="checkingId">
                    <SpinnerIcon class="animate-spin" />
                    {{ formatMessage(messages.checking, { item: reportItem }) }}...
                  </template>
                  <template v-else-if="checkedId && itemName">
                    <AutoLink
                      :to="itemLink"
                      target="_blank"
                      class="flex items-center gap-1 font-bold text-contrast hover:underline"
                    >
                      <Avatar
                        v-if="typeof itemIcon === 'string'"
                        :src="itemIcon"
                        :alt="itemName"
                        size="24px"
                        :circle="reportItem === 'user'"
                      />
                      <component :is="itemIcon" v-else-if="itemIcon" />
                      <span>{{ itemName }}</span>
                    </AutoLink>
                    <CheckIcon class="text-brand-green" />
                  </template>
                  <span v-else-if="checkedId" class="contents text-brand-red">
                    <IssuesIcon />
                    {{ formatMessage(messages.couldNotFind, { item: reportItem }) }}
                  </span>
                </div>
              </div>
            </div>
          </template>
          <template v-if="existingReport">
            {{
              formatMessage(messages.alreadyReportedDescription, { item: reportItem || "content" })
            }}
            <ButtonStyled color="brand">
              <nuxt-link :to="`/dashboard/report/${existingReport.id}`" class="w-fit">
                {{ formatMessage(messages.goToReport) }} <RightArrowIcon />
              </nuxt-link>
            </ButtonStyled>
          </template>
          <template v-else>
            <div class="flex flex-col gap-2" :class="{ hidden: !reportItemID }">
              <span class="text-lg font-bold text-contrast">
                {{ formatMessage(messages.whatReportReason, { item: reportItem || "content" }) }}
              </span>
              <RadioButtons v-slot="{ item }" v-model="reportType" :items="reportTypes">
                {{ item === "copyright" ? "Reuploaded work" : capitalizeString(item) }}
              </RadioButtons>
            </div>
            <div
              v-if="warnings[reportType]"
              class="flex gap-2 rounded-xl border-2 border-solid border-brand-orange bg-highlight-orange p-4 text-contrast"
            >
              <IssuesIcon class="h-5 w-5 shrink-0 text-orange" />
              <div class="flex flex-col gap-2">
                <p
                  v-for="(warning, index) in warnings[reportType]"
                  :key="`warning-${reportType}-${index}`"
                  class="m-0 leading-tight"
                >
                  <IntlFormatted :message-id="warning">
                    <template #copyright-policy-link="{ children }">
                      <nuxt-link class="text-link" :to="`/legal/copyright`">
                        <component :is="() => children" />
                      </nuxt-link>
                    </template>
                  </IntlFormatted>
                </p>
              </div>
            </div>
            <div :class="{ hidden: !reportType }">
              <span class="text-lg font-bold text-contrast">
                {{ formatMessage(messages.reportBodyTitle) }}
              </span>
              <p class="m-0 leading-tight text-secondary">
                {{ formatMessage(messages.reportBodyDescription) }}
              </p>
            </div>
            <div :class="{ hidden: !reportType }">
              <MarkdownEditor
                v-model="reportBody"
                placeholder=""
                :on-image-upload="onImageUpload"
              />
            </div>
            <div :class="{ hidden: !reportType }">
              <ButtonStyled color="brand">
                <button
                  id="submit-button"
                  :disabled="submitLoading || !canSubmit"
                  @click="submitReport"
                >
                  <SendIcon aria-hidden="true" />
                  {{ formatMessage(messages.submitReport) }}
                </button>
              </ButtonStyled>
            </div>
          </template>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  MarkdownEditor,
  RadialHeader,
  RadioButtons,
  ButtonStyled,
  Avatar,
  AutoLink,
} from "@modrinth/ui";
import {
  ExternalIcon,
  LeftArrowIcon,
  RightArrowIcon,
  CheckIcon,
  SpinnerIcon,
  SendIcon,
  IssuesIcon,
  CheckCircleIcon,
  XCircleIcon,
  ScaleIcon,
  VersionIcon,
} from "@modrinth/assets";
import type { User, Version, Report } from "@modrinth/utils";
import { useVIntl, defineMessages, type MessageDescriptor } from "@vintl/vintl";
import { useImageUpload } from "~/composables/image-upload.ts";

const tags = useTags();
const route = useNativeRoute();
const router = useRouter();

const auth = await useAuth();
const { formatMessage } = useVIntl();

if (!auth.value.user) {
  router.push("/auth/sign-in?redirect=" + encodeURIComponent(route.fullPath));
}

const accessQuery = (id: string): string => {
  return route.query?.[id]?.toString() || "";
};

const submitLoading = ref<boolean>(false);

const uploadedImageIDs = ref<string[]>([]);

const reportBody = ref<string>(accessQuery("body"));
const reportItem = ref<string>(accessQuery("item"));
const reportItemID = ref<string>(accessQuery("itemID"));
const reportType = ref<string>("");

const prefilled = ref<boolean>(!!reportItem.value && !!reportItemID.value);
const checkedId = ref<boolean>(false);
const checkingId = ref<boolean>(false);

const currentProject = ref<Project | null>(null);
const currentVersion = ref<Version | null>(null);
const currentUser = ref<User | null>(null);

const itemIcon = ref<string | Component | undefined>();
const itemName = ref<string | undefined>();
const itemLink = ref<string | undefined>();
const itemId = ref<string | undefined>();
const itemIssueTracker = ref<string | undefined>();

const reports = ref<Report[]>([]);
const existingReport = computed(() =>
  reports.value.find(
    (x) =>
      (x.item_id === reportItemID.value || x.item_id === itemId.value) &&
      x.item_type === reportItem.value,
  ),
);

await fetchItem();
await fetchExistingReports();

const currentItemValid = computed(
  () => !!currentProject.value || !!currentVersion.value || !!currentUser.value,
);

async function fetchExistingReports() {
  reports.value = ((await useBaseFetch("report?count=1000")) as Report[]).filter(
    (x) => x.reporter === auth.value.user?.id,
  );
}

async function fetchItem() {
  if (reportItem.value && reportItemID.value) {
    checkingId.value = true;
    itemIcon.value = undefined;
    itemName.value = undefined;
    itemLink.value = undefined;
    itemId.value = undefined;
    itemIssueTracker.value = undefined;
    try {
      if (reportItem.value === "project") {
        const project = (await useBaseFetch(`project/${reportItemID.value}`)) as Project;
        currentProject.value = project;

        itemIcon.value = project.icon_url;
        itemName.value = project.title;
        itemLink.value = `/project/${project.id}`;
        itemId.value = project.id;
        itemIssueTracker.value = project.issues_url;
      } else if (reportItem.value === "version") {
        const version = (await useBaseFetch(`version/${reportItemID.value}`)) as Version;
        currentVersion.value = version;

        itemIcon.value = VersionIcon;
        itemName.value = version.version_number;
        itemLink.value = `project/${version.project_id}/version/${version.id}`;
        itemId.value = version.id;
      } else if (reportItem.value === "user") {
        const user = (await useBaseFetch(`user/${reportItemID.value}`)) as User;
        currentUser.value = user;

        itemIcon.value = user.avatar_url;
        itemName.value = user.username;
        itemLink.value = `/user/${user.username}`;
        itemId.value = user.id;
      }
    } catch {}
    checkedId.value = true;
    checkingId.value = false;
  }
}

const reportItems = ["project", "version", "user"];
const reportTypes = computed(() => tags.value.reportTypes);

const canSubmit = computed(() => {
  return (
    reportItem.value !== "" &&
    reportItemID.value !== "" &&
    reportType.value !== "" &&
    reportBody.value !== ""
  );
});

const submissionValidation = () => {
  if (!canSubmit.value) {
    throw new Error("Please fill out all required fields");
  }

  if (reportItem.value === "") {
    throw new Error("Please select a report item");
  }

  if (reportItemID.value === "") {
    throw new Error("Please enter a report item ID");
  }

  if (reportType.value === "") {
    throw new Error("Please select a report type");
  }

  if (reportBody.value === "") {
    throw new Error("Please enter a report body");
  }

  return true;
};

const capitalizeString = (value?: string) => {
  if (!value) return "";
  return value?.charAt(0).toUpperCase() + value?.slice(1);
};

const submitReport = async () => {
  submitLoading.value = true;

  let data: {
    [key: string]: unknown;
  } = {
    report_type: reportType.value,
    item_type: reportItem.value,
    item_id: reportItemID.value,
    body: reportBody.value,
  };

  function takeNLast<T>(arr: T[], n: number): T[] {
    return arr.slice(Math.max(arr.length - n, 0));
  }

  if (uploadedImageIDs.value.length > 0) {
    data = {
      ...data,
      uploaded_images: takeNLast(uploadedImageIDs.value, 10),
    };
  }

  try {
    submissionValidation();
  } catch (error) {
    submitLoading.value = false;

    if (error instanceof Error) {
      addNotification({
        group: "main",
        title: "An error occurred",
        text: error.message,
        type: "error",
      });
    }

    return;
  }

  try {
    const response = (await useBaseFetch("report", {
      method: "POST",
      body: data,
    })) as { id: string };

    submitLoading.value = false;

    if (response?.id) {
      navigateTo(`/dashboard/report/${response.id}`);
    }
  } catch (error) {
    submitLoading.value = false;

    if (error instanceof Error) {
      addNotification({
        group: "main",
        title: "An error occurred",
        text: error.message,
        type: "error",
      });
    }

    throw error;
  }
};

const onImageUpload = async (file: File) => {
  const item = await useImageUpload(file, { context: "report" });
  uploadedImageIDs.value.push(item.id);
  return item.url;
};

const warnings: Record<string, MessageDescriptor[]> = {
  copyright: [
    defineMessage({
      id: "report.note.copyright.1",
      defaultMessage:
        "Please note that you are *not* submitting a DMCA takedown request, but rather a report of reuploaded content.",
    }),
    defineMessage({
      id: "report.note.copyright.2",
      defaultMessage:
        "If you meant to file a DMCA takedown request (which is a legal action) instead, please see our <copyright-policy-link>Copyright Policy</copyright-policy-link>.",
    }),
  ],
  malicious: [
    defineMessage({
      id: "report.note.malicious.1",
      defaultMessage:
        "Reports for malicious or deceptive content must include substantial evidence of the behavior, such as code samples.",
    }),
    defineMessage({
      id: "report.note.malicious.2",
      defaultMessage:
        "Summaries from Microsoft Defender, VirusTotal, or AI malware detection are not sufficient forms of evidence and will not be accepted.",
    }),
  ],
};

const messages = defineMessages({
  reportContent: {
    id: "report.report-content",
    defaultMessage: "Report content to moderators",
  },
  reportItem: {
    id: "report.report-item",
    defaultMessage: "Report {title} to moderators",
  },
  alreadyReportedItem: {
    id: "report.already-reported",
    defaultMessage: "You've already reported {title}",
  },
  alreadyReportedDescription: {
    id: "report.already-reported-description",
    defaultMessage:
      "You have an open report for this {item} already. You can add more details to your report if you have more information to add.",
  },
  backToItem: {
    id: "report.back-to-item",
    defaultMessage: "Back to {item}",
  },
  goToReport: {
    id: "report.go-to-report",
    defaultMessage: "Go to report",
  },
  pleaseReport: {
    id: "report.please-report",
    defaultMessage: "Please report:",
  },
  formNotFor: {
    id: "report.form-not-for",
    defaultMessage: "This form is not for:",
  },
  violation: {
    id: "report.for.violation",
    defaultMessage:
      "Violation of Modrinth <rules-link>Rules</rules-link> or <terms-link>Terms of Use</terms-link>",
  },
  violationDescription: {
    id: "report.for.violation.description",
    defaultMessage:
      "Examples include malicious, spam, offensive, deceptive, misleading, and illegal content.",
  },
  bugReports: {
    id: "report.not-for.bug-reports",
    defaultMessage: "Bug reports",
  },
  bugReportsDescription: {
    id: "report.not-for.bug-reports.description",
    defaultMessage: "You can report bugs to their <issues-link>issue tracker</issues-link>.",
  },
  dmcaTakedown: {
    id: "report.not-for.dmca",
    defaultMessage: "DMCA takedowns",
  },
  dmcaTakedownDescription: {
    id: "report.not-for.dmca.description",
    defaultMessage: "See our <policy-link>Copyright Policy</policy-link>.",
  },
  whatContentType: {
    id: "report.question.content-type",
    defaultMessage: "What type of content are you reporting?",
  },
  whatContentId: {
    id: "report.question.content-id",
    defaultMessage: "What is the ID of the {item}?",
  },
  whatReportReason: {
    id: "report.question.report-reason",
    defaultMessage: "Which of Modrinth's rules is this {item} violating?",
  },
  checking: {
    id: "report.checking",
    defaultMessage: "Checking {item}...",
  },
  couldNotFind: {
    id: "report.could-not-find",
    defaultMessage: "Could not find {item}",
  },
  reportBodyTitle: {
    id: "report.body.title",
    defaultMessage: "Please provide additional context about your report",
  },
  reportBodyDescription: {
    id: "report.body.description",
    defaultMessage:
      "Include links and images if possible and relevant. Empty or insufficient reports will be closed and ignored.",
  },
  submitReport: {
    id: "report.submit",
    defaultMessage: "Submit report",
  },
});
</script>

<style scoped lang="scss">
.page {
  padding: 1rem;
  margin-left: auto;
  margin-right: auto;
  max-width: 56rem;
}
</style>
