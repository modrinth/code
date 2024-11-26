<template>
  <div class="page">
    <Card>
      <div class="content">
        <div>
          <h1 class="card-title-adjustments">提交举报</h1>
          <div>
            <p>
              改装对每个人来说都应该是安全的，因此 BBSMC 严肃对待滥用和恶意行为。如果您发现违反我们
              <nuxt-link class="text-link" to="/legal2/terms">服务条款</nuxt-link> 或我们的
              <nuxt-link class="text-link" to="/legal2/rules">规则</nuxt-link>, 请在此处向我们举报。
            </p>
            <p>
              此表单仅用于向 BBSMC
              工作人员举报滥用或有害内容。对于与特定项目相关的错误，请使用该项目指定的问题链接或邮件。
            </p>
            <p>您的隐私对我们很重要；请放心，您的身份信息将被保密。</p>
          </div>
        </div>
        <div class="report-info-section">
          <div class="report-info-item">
            <label for="report-item">要举报的项目类型</label>
            <DropdownSelect
              id="report-item"
              v-model="reportItem"
              name="report-item"
              :options="reportItems"
              :display-name="capitalizeString"
              :multiple="false"
              :searchable="false"
              :show-no-results="false"
              :show-labels="false"
              placeholder="Choose report item"
            />
          </div>
          <div class="report-info-item">
            <label for="report-item-id">资源ID</label>
            <input
              id="report-item-id"
              v-model="reportItemID"
              type="text"
              placeholder="输入资源ID"
              autocomplete="off"
              :disabled="reportItem === ''"
            />
          </div>
          <div class="report-info-item">
            <label for="report-type">举报理由</label>
            <DropdownSelect
              id="report-type"
              v-model="reportType"
              name="report-type"
              :options="reportTypes"
              :multiple="false"
              :searchable="false"
              :show-no-results="false"
              :show-labels="false"
              :display-name="capitalizeString"
              placeholder="举报类型"
            />
          </div>
        </div>
        <div class="report-submission-section">
          <div>
            <p>
              请提供有关举报的其他背景信息。如有可以的话,请包含链接和图像
              <strong>未填写的举报将被关闭</strong>
            </p>
          </div>
          <MarkdownEditor v-model="reportBody" placeholder="" :on-image-upload="onImageUpload" />
        </div>
        <div class="submit-button">
          <Button
            id="submit-button"
            color="primary"
            :disabled="submitLoading || !canSubmit"
            @click="submitReport"
          >
            <SaveIcon aria-hidden="true" />
            提交
          </Button>
        </div>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { Card, Button, MarkdownEditor, DropdownSelect } from "@modrinth/ui";
import { SaveIcon } from "@modrinth/assets";
import { useImageUpload } from "~/composables/image-upload.ts";

const tags = useTags();
const route = useNativeRoute();

const accessQuery = (id: string): string => {
  return route.query?.[id]?.toString() || "";
};

const submitLoading = ref<boolean>(false);

const uploadedImageIDs = ref<string[]>([]);

const reportBody = ref<string>(accessQuery("body"));
const reportItem = ref<string>(accessQuery("item"));
const reportItemID = ref<string>(accessQuery("itemID"));
const reportType = ref<string>("");

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
    throw new Error("请填写所有必填字段");
  }

  if (reportItem.value === "") {
    throw new Error("请选择要举报的项目");
  }

  if (reportItemID.value === "") {
    throw new Error("请输入要举报的资源的ID");
  }

  if (reportType.value === "") {
    throw new Error("请选择举报类型");
  }

  if (reportBody.value === "") {
    throw new Error("请输入举报内容");
  }

  return true;
};

const capitalizeString = (value?: string) => {
  if (!value) return "";
  if (value === "project") {
    return "资源";
  }
  if (value === "version") {
    return "版本";
  }
  if (value === "user") {
    return "用户";
  }
  if (value === "spam") {
    return "垃圾邮件";
  }
  if (value === "copyright") {
    return "版权";
  }
  if (value === "inappropriate") {
    return "违法内容";
  }
  if (value === "malicious") {
    return "恶意内容";
  }
  if (value === "name-squatting") {
    return "抢注";
  }
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
        title: "发生错误",
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
        title: "发生错误",
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
</script>

<style scoped lang="scss">
.submit-button {
  display: flex;
  justify-content: flex-end;
  width: 100%;

  margin-top: var(--spacing-card-md);
}

.card-title-adjustments {
  margin-block: var(--spacing-card-md) var(--spacing-card-sm);
}

.page {
  padding: 0.5rem;
  margin-left: auto;
  margin-right: auto;
  max-width: 56rem;
}

.content {
  // TODO: Get rid of this hack when removing global styles from the website.
  // Overflow decides the behavior of md editor but also clips the border.
  // In the future, we should use ring instead of block-shadow for the
  // green ring around the md editor
  padding-inline: var(--gap-md);
  padding-bottom: var(--gap-md);
  margin-inline: calc(var(--gap-md) * -1);

  display: grid;

  // Disable horizontal stretch
  grid-template-columns: minmax(0, 1fr);
  overflow: hidden;
}

.report-info-section {
  display: block;

  width: 100%;
  gap: var(--gap-md);

  :global(.animated-dropdown) {
    & > .selected {
      height: 40px;
    }
  }

  .report-info-item {
    display: block;

    width: 100%;
    max-width: 100%;

    label {
      display: block;
      margin-bottom: var(--gap-sm);
      color: var(--color-text-dark);
      font-size: var(--font-size-md);
      font-weight: var(--font-weight-bold);
      margin-block: var(--spacing-card-md) var(--spacing-card-sm);
    }
  }
}
</style>
