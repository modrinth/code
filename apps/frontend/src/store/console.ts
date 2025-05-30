import { createGlobalState } from "@vueuse/core";
import { type Ref, shallowRef } from "vue";

/**
 * Maximum number of console output lines to store
 * @type {number}
 */
const maxLines = 10000;
const batchTimeout = 300; // ms
const initialBatchSize = 256;

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
  const output: Ref<string[]> = shallowRef<string[]>([]);
  const searchQuery: Ref<string> = shallowRef("");
  const filteredOutput: Ref<string[]> = shallowRef([]);
  let searchRegex: RegExp | null = null;

  let lineBuffer: string[] = [];
  let batchTimer: NodeJS.Timeout | null = null;
  let isProcessingInitialBatch = false;

  let refilterTimer: NodeJS.Timeout | null = null;
  const refilterTimeout = 100; // ms

  const updateFilter = () => {
    if (!searchQuery.value) {
      filteredOutput.value = [];
      return;
    }

    if (!searchRegex) {
      searchRegex = new RegExp(searchQuery.value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");
    }

    filteredOutput.value = output.value.filter((line) => searchRegex?.test(line) ?? false);
  };

  const scheduleRefilter = () => {
    if (refilterTimer) clearTimeout(refilterTimer);
    refilterTimer = setTimeout(updateFilter, refilterTimeout);
  };

  const flushBuffer = () => {
    if (lineBuffer.length === 0) return;

    const processedLines = lineBuffer.flatMap((line) => line.split("\n").filter(Boolean));

    if (isProcessingInitialBatch && processedLines.length >= initialBatchSize) {
      isProcessingInitialBatch = false;
      output.value = processedLines.slice(-maxLines);
    } else {
      const newOutput = [...output.value, ...processedLines];
      output.value = newOutput.slice(-maxLines);
    }

    lineBuffer = [];
    batchTimer = null;

    if (searchQuery.value) {
      scheduleRefilter();
    }
  };

  /**
   * Adds a new output line to the console output
   * Automatically removes the oldest line if max output is exceeded
   *
   * @param {string} line - The console output line to add
   */
  const addLine = (line: string): void => {
    lineBuffer.push(line);

    if (!batchTimer) {
      batchTimer = setTimeout(flushBuffer, batchTimeout);
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
    if (output.value.length === 0 && lines.length >= initialBatchSize) {
      isProcessingInitialBatch = true;
      lineBuffer = lines;
      flushBuffer();
      return;
    }

    lineBuffer.push(...lines);

    if (!batchTimer) {
      batchTimer = setTimeout(flushBuffer, batchTimeout);
    }
  };

  /**
   * Sets the search query and filters the output based on the query
   *
   * @param {string} query - The search query
   */
  const setSearchQuery = (query: string): void => {
    searchQuery.value = query;
    searchRegex = null;
    updateFilter();
  };

  /**
   * Clears all console output lines
   */
  const clear = (): void => {
    output.value = [];
    filteredOutput.value = [];
    searchQuery.value = "";
    lineBuffer = [];
    isProcessingInitialBatch = false;
    if (batchTimer) {
      clearTimeout(batchTimer);
      batchTimer = null;
    }
    if (refilterTimer) {
      clearTimeout(refilterTimer);
      refilterTimer = null;
    }
    searchRegex = null;
  };

  /**
   * Finds the index of a line in the main output
   *
   * @param {string} line - The line to find
   * @returns {number} The index of the line, or -1 if not found
   */
  const findLineIndex = (line: string): number => {
    return output.value.findIndex((l) => l === line);
  };

  return {
    output,
    searchQuery,
    filteredOutput,
    addLine,
    addLines,
    setSearchQuery,
    clear,
    findLineIndex,
  };
});
