<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let {
    dbName,
    onClose,
  }: {
    dbName: string;
    onClose: () => void;
  } = $props();

  let logs = $state<string>("");
  let autoRefreshLogs = $state<boolean>(false);
  let logsRefreshInterval: ReturnType<typeof setInterval> | null = null;

  async function fetchLogs() {
    try {
      const result = await invoke<string>("get_container_logs", {
        dbName: dbName,
        tailLines: 200,
      });
      logs = result;
    } catch (error) {
      logs = `Error fetching logs: ${error}`;
    }
  }

  $effect(() => {
    fetchLogs();

    if (autoRefreshLogs) {
      logsRefreshInterval = setInterval(() => {
        fetchLogs();
      }, 2000);
    } else if (logsRefreshInterval) {
      clearInterval(logsRefreshInterval);
      logsRefreshInterval = null;
    }

    return () => {
      if (logsRefreshInterval) {
        clearInterval(logsRefreshInterval);
        logsRefreshInterval = null;
      }
    };
  });
</script>

<div
  class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-8"
  onclick={onClose}
>
  <div
    class="bg-white dark:bg-gray-800 rounded-xl w-full max-w-4xl max-h-[85vh] flex flex-col shadow-2xl"
    onclick={(e) => e.stopPropagation()}
  >
    <div
      class="flex justify-between items-center px-6 py-4 border-b-2 border-gray-200 dark:border-gray-700"
    >
      <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
        {dbName} Logs
      </h2>
      <div class="flex gap-4 items-center">
        <label
          class="flex items-center gap-2 text-gray-900 dark:text-gray-100 cursor-pointer text-sm"
        >
          <input type="checkbox" bind:checked={autoRefreshLogs} class="cursor-pointer" />
          Auto-refresh
        </label>
        <button
          onclick={fetchLogs}
          class="px-4 py-2 bg-blue-500 text-white rounded-md font-semibold text-sm hover:bg-blue-600 transition-colors"
        >
          🔄 Refresh
        </button>
        <button
          onclick={onClose}
          class="px-4 py-2 bg-red-500 text-white rounded-md font-bold text-xl hover:bg-red-600 transition-colors"
        >
          ✕
        </button>
      </div>
    </div>
    <div class="flex-1 overflow-auto px-6 py-4 bg-gray-50 dark:bg-gray-900">
      <pre
        class="m-0 p-4 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono text-sm leading-relaxed rounded-md border border-gray-200 dark:border-gray-700 whitespace-pre-wrap break-words">{logs}</pre>
    </div>
  </div>
</div>
