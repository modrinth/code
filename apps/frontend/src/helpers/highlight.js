import hljs from "highlight.js/lib/core";
// Scripting
import javascript from "highlight.js/lib/languages/javascript";
import lua from "highlight.js/lib/languages/lua";
import python from "highlight.js/lib/languages/python";
// Coding
import groovy from "highlight.js/lib/languages/groovy";
import java from "highlight.js/lib/languages/java";
import kotlin from "highlight.js/lib/languages/kotlin";
import scala from "highlight.js/lib/languages/scala";
// Configs
import { configuredXss, md } from "@modrinth/utils";
import gradle from "highlight.js/lib/languages/gradle";
import ini from "highlight.js/lib/languages/ini";
import json from "highlight.js/lib/languages/json";
import properties from "highlight.js/lib/languages/properties";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";

/* REGISTRATION */
// Scripting
hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("python", python);
hljs.registerLanguage("lua", lua);
// Coding
hljs.registerLanguage("java", java);
hljs.registerLanguage("kotlin", kotlin);
hljs.registerLanguage("scala", scala);
hljs.registerLanguage("groovy", groovy);
// Configs
hljs.registerLanguage("gradle", gradle);
hljs.registerLanguage("json", json);
hljs.registerLanguage("ini", ini);
hljs.registerLanguage("yaml", yaml);
hljs.registerLanguage("xml", xml);
hljs.registerLanguage("properties", properties);

/* ALIASES */
// Scripting
hljs.registerAliases(["js"], { languageName: "javascript" });
hljs.registerAliases(["py"], { languageName: "python" });
// Coding
hljs.registerAliases(["kt"], { languageName: "kotlin" });
// Configs
hljs.registerAliases(["json5"], { languageName: "json" });
hljs.registerAliases(["toml"], { languageName: "ini" });
hljs.registerAliases(["yml"], { languageName: "yaml" });
hljs.registerAliases(["html", "htm", "xhtml", "mcui", "fxml"], { languageName: "xml" });

export const renderHighlightedString = (string) =>
  configuredXss.process(
    md({
      highlight: function (str, lang) {
        if (lang && hljs.getLanguage(lang)) {
          try {
            return hljs.highlight(str, { language: lang }).value;
          } catch {
            /* empty */
          }
        }

        return "";
      },
    }).render(string),
  );
