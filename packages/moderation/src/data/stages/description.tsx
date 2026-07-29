import {LibraryIcon} from "@modrinth/assets"
import {injectProjectPageContext} from "@modrinth/ui"
import {computed} from "vue"

import {group, markdown, md, stage, toggle} from "../../types/node"

const SHOW_SPOILER_ADVICE = ["spoilers"]

export default function () {
  const {projectV3: project} = injectProjectPageContext()

  return stage("description", "Description")
    .hint("Is the description sufficient, accurate, and accessible?")
    .guidance(
      "https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080508042e70089dd787e",
    )
    .icon(LibraryIcon)
    .navigate()
    .children(
      group()
        .title("Description Issues?")
        .children(
          toggle("insufficient", "Insufficient")
            .suggestedStatus("flagged")
            .message("insufficient/header")
            .children(
              group()
                .title("Why is this Description Insufficient?")
                .children(
                  toggle("custom", "Custom").children(
                    markdown("explainer")
                      .title("How can the author improve their description?")
                      .required()
                      .rawMessage((state) => `${state.value}\n\n`),
                  ),
                  toggle("fork", "Fork").message("piece/fork"),
                  toggle("unfinished", "Unfinished").message("piece/unfinished"),
                  toggle("spoilers", "Spoilers").message("piece/spoilers"),
                ),
            )
            .collect(
              () =>
                `insufficient/default/${project.value?.minecraft_java_server ? "servers" : project.value?.project_types?.includes("modpack") ? "packs" : "projects"}`,
            )
            .rawMessage(async (state) => {
              return SHOW_SPOILER_ADVICE.some((reason) => state?.[reason] === true)
                ? await md("checklist/messages/description/insufficient/piece/spoiler-guide")(state)
                : ""
            }),

          toggle("non-english", "Non-english")
            .suggestedStatus("flagged")
            .message(() => `non-english${project.value.minecraft_java_server ? "-server" : ""}`)
            .shown(
              computed(() => {
                return !(
                  !!project.value?.minecraft_java_server &&
                  !project.value.minecraft_server?.languages?.includes("en")
                )
              }),
            ),

          toggle("headers-as-body", "Headers as body text")
            .suggestedStatus("flagged")
            .message(),

          toggle("image-only", "Image-only")
            .suggestedStatus("flagged")
            .message(),

          toggle("non-standard-text", "Non-standard text")
            .suggestedStatus("flagged")
            .message(),

          toggle("clarity", "Unclear / Misleading")
            .suggestedStatus("rejected")
            .message(),
        ),
    )
}
