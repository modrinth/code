import { createGlobalState } from "@vueuse/core";
import { type Ref, ref } from "vue";

/**
 * Maximum number of console output lines to store
 * @type {number}
 */
const maxLines = 5000;

/**
 * Provides a global console output state management system
 * Allows adding, storing, and clearing console output with a maximum line limit
 *
 * @returns {Object} Console state management methods and reactive state
 * @property {Ref<string[]>} consoleOutput - Reactive array of console output lines
 * @property {function(string): void} addConsoleOutput - Method to add a new console output line
 * @property {function(): void} clear - Method to clear all console output
 */
export const usePyroConsole = createGlobalState(() => {
  /**
   * Reactive array storing console output lines
   * @type {Ref<string[]>}
   */
  const output: Ref<string[]> = ref<string[]>([]);

  /**
   * Adds a new output line to the console output
   * Automatically removes the oldest line if max output is exceeded
   *
   * @param {string} line - The console output line to add
   */
  const addLine = (line: string): void => {
    output.value.push(line);

    if (output.value.length > maxLines) {
      output.value.shift();
    }
  };

  /**
   * Adds multiple output lines to the console output
   * Automatically removes the oldest lines if max output is exceeded
   *
   * @param {string[]} lines - The console output lines to add
   * @returns {void}
   */
  const addLines = (lines: string[]): void => {
    output.value.push(...lines);

    if (output.value.length > maxLines) {
      output.value.splice(0, output.value.length - maxLines);
    }
  };

  /**
   * Clears all console output lines
   */
  const clear = (): void => {
    output.value = [];
  };

  return {
    output,
    addLine,
    addLines,
    clear,
  };
});
