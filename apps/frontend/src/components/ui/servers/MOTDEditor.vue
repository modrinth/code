<template>
  <div class="flex flex-col gap-4 rounded-xl bg-bg p-4">
    <div class="mcbg flex gap-4 p-4 text-white">
      <slot />
      <div class="font-minecraft mb-2 flex flex-col gap-1 text-2xl">
        Minecraft Server
        <div
          ref="editor"
          contenteditable
          @input="handleInput"
          @keyup="updateCursorPosition"
          @click="updateCursorPosition"
          class="min-h-[50px] whitespace-pre-wrap outline-none"
          spellcheck="false"
          v-html="formattedText"
        ></div>
      </div>
    </div>
    <div class="mb-2 flex items-center justify-between">
      <div class="flex flex-wrap gap-1 rounded-lg bg-button-bg p-2">
        <div
          v-for="format in formatCodes"
          :key="format.code"
          @click="insertFormatCode(format.code)"
          :class="[`h-6 w-6 rounded-full p-0 hover:opacity-80`, format.color]"
          v-tooltip="`${format.description}`"
        ></div>
      </div>
      <div class="flex gap-1">
        <Button
          v-for="style in styleButtons"
          :key="style.code"
          @click="insertFormatCode(style.code)"
          class="h-6 w-6 rounded bg-white p-0"
          v-tooltip="style.description"
          icon-only
        >
          <component :is="style.icon" class="h-4 w-4" />
        </Button>
        <Button
          @click="openHexColorPicker"
          class="from-red-500 via-green-500 to-blue-500 h-6 w-6 rounded bg-gradient-to-r p-0"
          v-tooltip="'Custom Hex Color'"
          icon-only
        >
          <PaintBrushIcon class="h-4 w-4" />
        </Button>
        <Button
          @click="openEmojiPicker"
          class="flex h-6 w-6 items-center justify-center rounded bg-white p-0"
          v-tooltip="'Minecraft Emoji'"
          icon-only
        >
          ☺
        </Button>
      </div>
    </div>
    {{ motd }}

    <!-- Hex Color Picker Modal -->
    <div
      v-if="showHexColorPicker"
      class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50"
    >
      <div class="rounded-lg bg-white p-4">
        <h3 class="mb-2 text-lg font-semibold">Choose a Hex Color</h3>
        <input type="color" v-model="customHexColor" class="mb-2" />
        <div>
          <button @click="applyHexColor" class="bg-blue-500 mr-2 rounded px-4 py-2 text-white">
            Apply
          </button>
          <button @click="showHexColorPicker = false" class="rounded bg-gray-300 px-4 py-2">
            Cancel
          </button>
        </div>
      </div>
    </div>

    <!-- Emoji Picker Modal -->
    <div
      v-if="showEmojiPicker"
      class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50"
    >
      <div class="rounded-lg bg-white p-4">
        <h3 class="mb-2 text-lg font-semibold">Choose a Minecraft Emoji</h3>
        <div class="grid max-h-96 grid-cols-8 gap-2 overflow-y-auto">
          <button
            v-for="emoji in minecraftEmojis"
            :key="emoji.char"
            @click="insertEmoji(emoji.char)"
            class="rounded p-2 text-2xl hover:bg-gray-200"
            :title="emoji.name"
          >
            {{ emoji.char }}
          </button>
        </div>
        <button @click="showEmojiPicker = false" class="mt-4 rounded bg-gray-300 px-4 py-2">
          Close
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  BoldIcon,
  ClearIcon,
  ItalicIcon,
  PaintBrushIcon,
  StrikethroughIcon,
  UnderlineIcon,
} from "@modrinth/assets";
import Button from "@modrinth/ui/src/components/base/Button.vue";
import { ref, computed, onMounted } from "vue";

const formatCodes = [
  { code: "§f", color: "bg-white", description: "White" },
  { code: "§7", color: "bg-[#AAAAAA]", description: "Gray" },
  { code: "§8", color: "bg-[#555555]", description: "Dark Gray" },
  { code: "§0", color: "bg-[#000000]", description: "Black" },
  { code: "§9", color: "bg-[#5555FF]", description: "Blue" },
  { code: "§1", color: "bg-[#0000AA]", description: "Dark Blue" },
  { code: "§b", color: "bg-[#55FFFF]", description: "Aqua" },
  { code: "§3", color: "bg-[#00AAAA]", description: "Dark Aqua" },
  { code: "§a", color: "bg-[#55FF55]", description: "Green" },
  { code: "§2", color: "bg-[#00AA00]", description: "Dark Green" },
  { code: "§e", color: "bg-[#FFFF55]", description: "Yellow" },
  { code: "§6", color: "bg-[#FFAA00]", description: "Gold" },
  { code: "§c", color: "bg-[#FF5555]", description: "Red" },
  { code: "§4", color: "bg-[#AA0000]", description: "Dark Red" },
  { code: "§d", color: "bg-[#FF55FF]", description: "Light Purple" },
  { code: "§5", color: "bg-[#AA00AA]", description: "Dark Purple" },
];

const styleButtons = [
  { code: "§l", icon: BoldIcon, description: "Bold" },
  { code: "§o", icon: ItalicIcon, description: "Italic" },
  { code: "§n", icon: UnderlineIcon, description: "Underline" },
  { code: "§m", icon: StrikethroughIcon, description: "Strikethrough" },
  { code: "§r", icon: ClearIcon, description: "Reset" },
];

const colorMap = {
  "§0": "#000000",
  "§1": "#0000AA",
  "§2": "#00AA00",
  "§3": "#00AAAA",
  "§4": "#AA0000",
  "§5": "#AA00AA",
  "§6": "#FFAA00",
  "§7": "#AAAAAA",
  "§8": "#555555",
  "§9": "#5555FF",
  "§a": "#55FF55",
  "§b": "#55FFFF",
  "§c": "#FF5555",
  "§d": "#FF55FF",
  "§e": "#FFFF55",
};

const minecraftEmojis = [
  { char: "☺", name: "SMILING FACE" },
  { char: "☹", name: "FROWNING FACE" },
  { char: "☠", name: "SKULL AND CROSSBONES" },
  { char: "❣", name: "HEART EXCLAMATION" },
  { char: "❤", name: "RED HEART" },
  { char: "✌", name: "VICTORY HAND" },
  { char: "☝", name: "INDEX POINTING UP" },
  { char: "✍", name: "WRITING HAND" },
  { char: "♨", name: "HOT SPRINGS" },
  { char: "✈", name: "AIRPLANE" },
  { char: "⌛", name: "HOURGLASS DONE" },
  { char: "⌚", name: "WATCH" },
  { char: "☀", name: "SUN" },
  { char: "☁", name: "CLOUD" },
  { char: "☂", name: "UMBRELLA" },
  { char: "❄", name: "SNOWFLAKE" },
  { char: "☃", name: "SNOWMAN" },
  { char: "☄", name: "COMET" },
  { char: "♠", name: "SPADE SUIT" },
  { char: "♥", name: "HEART SUIT" },
  { char: "♦", name: "DIAMOND SUIT" },
  { char: "♣", name: "CLUB SUIT" },
  { char: "♟", name: "CHESS PAWN" },
  { char: "☎", name: "TELEPHONE" },
  { char: "⌨", name: "KEYBOARD" },
  { char: "✉", name: "ENVELOPE" },
  { char: "✏", name: "PENCIL" },
  { char: "✒", name: "BLACK PEN" },
  { char: "✂", name: "SCISSORS" },
  { char: "☢", name: "RADIOACTIVE" },
  { char: "☣", name: "BIOHAZARD" },
  { char: "⬆", name: "UP ARROW" },
  { char: "⬇", name: "DOWN ARROW" },
  { char: "➡", name: "RIGHT ARROW" },
  { char: "⬅", name: "LEFT ARROW" },
  { char: "↗", name: "UP-RIGHT ARROW" },
  { char: "↘", name: "DOWN-RIGHT ARROW" },
  { char: "↙", name: "DOWN-LEFT ARROW" },
  { char: "↖", name: "UP-LEFT ARROW" },
  { char: "↕", name: "UP-DOWN ARROW" },
  { char: "↔", name: "LEFT-RIGHT ARROW" },
  { char: "↩", name: "RIGHT ARROW CURVING LEFT" },
  { char: "↪", name: "LEFT ARROW CURVING RIGHT" },
  { char: "✡", name: "STAR OF DAVID" },
  { char: "☸", name: "WHEEL OF DHARMA" },
  { char: "☯", name: "YIN YANG" },
  { char: "✝", name: "LATIN CROSS" },
  { char: "☦", name: "ORTHODOX CROSS" },
  { char: "☪", name: "STAR AND CRESCENT" },
  { char: "☮", name: "PEACE SYMBOL" },
  { char: "♈", name: "ARIES" },
  { char: "♉", name: "TAURUS" },
  { char: "♊", name: "GEMINI" },
  { char: "♋", name: "CANCER" },
  { char: "♌", name: "LEO" },
  { char: "♍", name: "VIRGO" },
  { char: "♎", name: "LIBRA" },
  { char: "♏", name: "SCORPIO" },
  { char: "♐", name: "SAGITTARIUS" },
  { char: "♑", name: "CAPRICORN" },
  { char: "♒", name: "AQUARIUS" },
  { char: "♓", name: "PISCES" },
  { char: "▶", name: "PLAY BUTTON" },
  { char: "◀", name: "REVERSE BUTTON" },
  { char: "♀", name: "FEMALE SIGN" },
  { char: "♂", name: "MALE SIGN" },
  { char: "✖", name: "MULTIPLY" },
  { char: "‼", name: "DOUBLE EXCLAMATION MARK" },
  { char: "〰", name: "WAVY DASH" },
  { char: "☑", name: "CHECK BOX WITH CHECK" },
  { char: "✔", name: "CHECK MARK" },
  { char: "✳", name: "EIGHT-SPOKED ASTERISK" },
  { char: "✴", name: "EIGHT-POINTED STAR" },
  { char: "❇", name: "SPARKLE" },
  { char: "©", name: "COPYRIGHT" },
  { char: "®", name: "REGISTERED" },
  { char: "™", name: "TRADE MARK" },
  { char: "Ⓜ", name: "CIRCLED M" },
  { char: "㊗", name: 'JAPANESE "CONGRATULATIONS" BUTTON' },
  { char: "㊙", name: 'JAPANESE "SECRET" BUTTON' },
  { char: "▪", name: "BLACK SMALL SQUARE" },
  { char: "▫", name: "WHITE SMALL SQUARE" },
  { char: "☷", name: "TRIGRAM FOR EARTH" },
  { char: "☵", name: "TRIGRAM FOR WATER" },
  { char: "☶", name: "TRIGRAM FOR MOUNTAIN" },
  { char: "☋", name: "DESCENDING NODE" },
  { char: "☌", name: "CONJUNCTION" },
  { char: "♜", name: "BLACK CHESS ROOK" },
  { char: "♕", name: "WHITE CHESS QUEEN" },
  { char: "♡", name: "WHITE HEART SUIT" },
  { char: "♬", name: "BEAMED SIXTEENTH NOTES" },
  { char: "☚", name: "BLACK LEFT POINTING INDEX" },
  { char: "♮", name: "MUSIC NATURAL SIGN" },
  { char: "♝", name: "BLACK CHESS BISHOP" },
  { char: "♯", name: "SHARP" },
  { char: "☴", name: "TRIGRAM FOR WIND" },
  { char: "♭", name: "FLAT" },
  { char: "☓", name: "SALTIRE" },
  { char: "☛", name: "BLACK RIGHT POINTING INDEX" },
  { char: "☭", name: "HAMMER AND SICKLE" },
  { char: "♢", name: "WHITE DIAMOND SUIT" },
  { char: "✐", name: "UPPER RIGHT PENCIL" },
  { char: "♖", name: "WHITE CHESS ROOK" },
  { char: "☈", name: "THUNDERSTORM" },
  { char: "☒", name: "BALLOT BOX WITH X" },
  { char: "★", name: "BLACK STAR" },
  { char: "♚", name: "BLACK CHESS KING" },
  { char: "♛", name: "BLACK CHESS QUEEN" },
  { char: "✎", name: "LOWER RIGHT PENCIL" },
  { char: "♪", name: "EIGHTH NOTE" },
  { char: "☰", name: "TRIGRAM FOR HEAVEN" },
  { char: "☽", name: "FIRST QUARTER MOON" },
  { char: "☡", name: "CAUTION SIGN" },
  { char: "☼", name: "WHITE SUN WITH RAYS" },
  { char: "♅", name: "URANUS" },
  { char: "☐", name: "BALLOT BOX" },
  { char: "☟", name: "WHITE DOWN POINTING INDEX" },
  { char: "❦", name: "FLORAL HEART" },
  { char: "☊", name: "ASCENDING NODE" },
  { char: "☍", name: "OPPOSITION" },
  { char: "☬", name: "ADI SHAKTI" },
  { char: "♧", name: "WHITE CLUB SUIT" },
  { char: "☫", name: "FARSI SYMBOL" },
  { char: "☱", name: "TRIGRAM FOR LAKE" },
  { char: "☾", name: "LAST QUARTER MOON" },
  { char: "☤", name: "CADUCEUS" },
  { char: "❧", name: "ROTATED FLORAL HEART BULLET" },
  { char: "♄", name: "SATURN" },
  { char: "♁", name: "EARTH" },
  { char: "♔", name: "WHITE CHESS KING" },
  { char: "❥", name: "ROTATED HEAVY BLACK HEART BULLET" },
  { char: "☥", name: "ANKH" },
  { char: "☻", name: "BLACK SMILING FACE" },
  { char: "♤", name: "WHITE SPADE SUIT" },
  { char: "♞", name: "BLACK CHESS KNIGHT" },
  { char: "♆", name: "NEPTUNE" },
  { char: "#", name: "HASH SIGN" },
  { char: "♃", name: "JUPITER" },
  { char: "♩", name: "QUARTER NOTE" },
  { char: "☇", name: "LIGHTNING" },
  { char: "☞", name: "WHITE RIGHT POINTING INDEX" },
  { char: "♫", name: "BEAMED EIGHTH NOTES" },
  { char: "☏", name: "WHITE TELEPHONE" },
  { char: "♘", name: "WHITE CHESS KNIGHT" },
  { char: "☧", name: "CHI RHO" },
  { char: "☉", name: "SUN" },
  { char: "♇", name: "PLUTO" },
  { char: "☩", name: "CROSS OF JERUSALEM" },
  { char: "♙", name: "WHITE CHESS PAWN" },
  { char: "☜", name: "WHITE LEFT POINTING INDEX" },
  { char: "☲", name: "TRIGRAM FOR FIRE" },
  { char: "☨", name: "CROSS OF LORRAINE" },
  { char: "♗", name: "WHITE CHESS BISHOP" },
  { char: "☳", name: "TRIGRAM FOR THUNDER" },
  { char: "⚔", name: "CROSSED SWORDS" },
  { char: "⚀", name: "DICE ONE" },
];

const motd = ref("§bSimpily Optimized §f♦ §aModrith Servers");
const editor = ref(null);
const showHexColorPicker = ref(false);
const showGradientPicker = ref(false);
const showEmojiPicker = ref(false);
const customHexColor = ref("#000000");
const gradientStart = ref("#000000");
const gradientEnd = ref("#ffffff");
const cursorPosition = ref(0);
const lastFormattedLength = ref(0);

const formattedText = computed(() => {
  return renderFormattedText(motd.value);
});

const insertFormatCode = (code) => {
  const newMotd =
    motd.value.slice(0, cursorPosition.value) + code + motd.value.slice(cursorPosition.value);
  motd.value = newMotd;
  cursorPosition.value += code.length;
  updateEditorContent();
};

const handleInput = (e) => {
  const selection = window.getSelection();
  const offset = selection.focusOffset;
  const node = selection.focusNode;

  // Calculate the actual cursor position in the raw text
  let actualPosition = 0;
  const traverse = (node) => {
    if (node === selection.focusNode) {
      actualPosition += offset;
      return true;
    }
    if (node.nodeType === Node.TEXT_NODE) {
      actualPosition += node.length;
    } else {
      for (let child of node.childNodes) {
        if (traverse(child)) {
          return true;
        }
      }
    }
    return false;
  };
  traverse(editor.value);

  // Convert formatted text position to raw text position
  const rawPosition = formattedToRawPosition(actualPosition);

  // Update motd while preserving formatting codes
  const newText = extractRawText(e.target.innerHTML);
  console.log(newText);
  motd.value = insertTextPreservingFormatting(motd.value, newText);

  // Update cursor position
  cursorPosition.value = rawPosition + (newText.length - extractRawText(motd.value).length);

  updateEditorContent();
};

const insertTextPreservingFormatting = (text, newText) => {
  const parts = text.split(/(§[0-9a-flmnor]|§x[0-9A-Fa-f]{6}|§#[0-9A-Fa-f]{6}|§g\{[^}]+\})/);
  let result = "";

  for (let part of parts) {
    if (part.startsWith("§")) {
      result += part;
    } else {
      result += newText.slice(0, part.length);
      newText = newText.slice(part.length);
    }
  }
  return result + newText;
};

const formattedToRawPosition = (formattedPos) => {
  let rawPos = 0;
  let formattedCount = 0;
  const parts = motd.value.split(/(§[0-9a-flmnor]|§x[0-9A-Fa-f]{6}|§#[0-9A-Fa-f]{6}|§g\{[^}]+\})/);

  for (let part of parts) {
    if (part.startsWith("§")) {
      rawPos += part.length;
    } else {
      if (formattedCount + part.length >= formattedPos) {
        rawPos += formattedPos - formattedCount;
        break;
      }
      rawPos += part.length;
      formattedCount += part.length;
    }
  }

  return rawPos;
};

const extractRawText = (html) => {
  const tempDiv = document.createElement("div");
  tempDiv.innerHTML = html;
  return tempDiv.textContent || tempDiv.innerText || "";
};

const updateEditorContent = () => {
  if (editor.value) {
    const formattedContent = renderFormattedText(motd.value);
    if (editor.value.innerHTML !== formattedContent) {
      editor.value.innerHTML = formattedContent;
    }
    nextTick(() => {
      setCaretPosition(editor.value, cursorPosition.value);
    });
  }
};

const setCaretPosition = (element, position) => {
  const range = document.createRange();
  const sel = window.getSelection();
  let currentLength = 0;
  let targetNode = null;
  let targetOffset = 0;

  const traverse = (node) => {
    if (node.nodeType === Node.TEXT_NODE) {
      const nodeLength = node.length;
      if (currentLength + nodeLength >= position) {
        targetNode = node;
        targetOffset = position - currentLength;
        return true;
      }
      currentLength += nodeLength;
    } else {
      for (let i = 0; i < node.childNodes.length; i++) {
        if (traverse(node.childNodes[i])) {
          return true;
        }
      }
    }
    return false;
  };

  traverse(element);

  if (targetNode) {
    range.setStart(targetNode, targetOffset);
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
  }
};

const renderFormattedText = (text) => {
  const parts = text.split(/(§[0-9a-flmnor]|§x[0-9A-Fa-f]{6}|§#[0-9A-Fa-f]{6}|§g\{[^}]+\})/);
  let currentStyle = "";
  let formattedText = "";

  parts.forEach((part) => {
    if (part.match(/^§[0-9a-f]$/)) {
      currentStyle = `color: ${colorMap[part]};` || "";
    } else if (part.match(/^§x[0-9A-Fa-f]{6}$/)) {
      const hexColor = part.slice(2);
      currentStyle = `color: #${hexColor};`;
    } else if (part.match(/^§#[0-9A-Fa-f]{6}$/)) {
      const hexColor = part.slice(2);
      currentStyle = `color: ${hexColor};`;
    } else if (part.match(/^§g\{[^}]+\}$/)) {
      const gradientColors = part.slice(3, -1).split(",");
      currentStyle = `background: linear-gradient(to right, ${gradientColors.join(", ")}); -webkit-background-clip: text; -webkit-text-fill-color: transparent;`;
    } else if (part === "§l") {
      currentStyle += " font-weight: bold;";
    } else if (part === "§m") {
      currentStyle += " text-decoration: line-through;";
    } else if (part === "§n") {
      currentStyle += " text-decoration: underline;";
    } else if (part === "§o") {
      currentStyle += " font-style: italic;";
    } else if (part === "§r") {
      currentStyle = "";
    } else {
      formattedText += `<span style="${currentStyle}">${part}</span>`;
    }
  });

  return formattedText;
};

const openHexColorPicker = () => {
  showHexColorPicker.value = true;
};

const applyHexColor = () => {
  insertFormatCode(`§#${customHexColor.value.slice(1)}`);
  showHexColorPicker.value = false;
};

const openGradientPicker = () => {
  showGradientPicker.value = true;
};

const applyGradient = () => {
  const gradientCode = `§g{${gradientStart.value},${gradientEnd.value}}`;
  insertFormatCode(gradientCode);
  showGradientPicker.value = false;
};

const openEmojiPicker = () => {
  showEmojiPicker.value = true;
};

const insertEmoji = (emoji) => {
  insertFormatCode(emoji);
  showEmojiPicker.value = false;
};

onMounted(() => {
  if (editor.value) {
    editor.value.innerHTML = renderFormattedText(motd.value);
  }
});
</script>

<style scoped>
@font-face {
  font-family: "Monocraft";
  src: url("/Monocraft.ttf") format("truetype");
}

.font-minecraft {
  font-family: "Monocraft", monospace;
}

.mcbg {
  background: url("@/assets/images/servers/minecraft-background-dark.png") repeat center center;
}
</style>
