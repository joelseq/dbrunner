<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ConnectionString from "./ConnectionString.svelte";

  interface DatabaseInfo {
    name: string;
    status: string;
    port: number;
    image: string;
    volume_path?: string;
  }

  interface CommandResult {
    success: boolean;
    message: string;
  }

  let {
    db = $bindable<DatabaseInfo>(),
    onMessage,
    onOpenLogs,
    onCopy,
  }: {
    db: DatabaseInfo;
    onMessage: (msg: string) => void;
    onOpenLogs: (db: DatabaseInfo) => void;
    onCopy: (text: string, dbName: string) => void;
  } = $props();

  let loading = $state(false);
  let editingVolumePath = $state(false);
  let newVolumePath = $state("");
  let editingImageTag = $state(false);
  let newImageTag = $state("");
  let showConnectionString = $state(false);

  async function startDatabase() {
    loading = true;
    onMessage(`Starting ${db.name}...`);

    try {
      const result = await invoke<CommandResult>("start_database", {
        dbName: db.name,
      });
      onMessage(result.message);

      if (result.success) {
        const status = await invoke<string>("get_database_status", {
          dbName: db.name,
        });
        db.status = status;
      }
    } catch (error) {
      onMessage(`Error: ${error}`);
    } finally {
      loading = false;
    }
  }

  async function stopDatabase() {
    loading = true;
    onMessage(`Stopping ${db.name}...`);

    try {
      const result = await invoke<CommandResult>("stop_database", {
        dbName: db.name,
      });
      onMessage(result.message);

      if (result.success) {
        const status = await invoke<string>("get_database_status", {
          dbName: db.name,
        });
        db.status = status;
      }
    } catch (error) {
      onMessage(`Error: ${error}`);
    } finally {
      loading = false;
    }
  }

  function startEditingPath() {
    editingVolumePath = true;
    newVolumePath = db.volume_path || "";
  }

  function cancelEditingPath() {
    editingVolumePath = false;
    newVolumePath = "";
  }

  async function saveVolumePath() {
    if (!newVolumePath) {
      onMessage("Please enter a valid path");
      return;
    }

    try {
      const result = await invoke<CommandResult>("set_volume_path", {
        dbName: db.name,
        path: newVolumePath,
      });

      onMessage(result.message);

      if (result.success) {
        db.volume_path = newVolumePath;
        editingVolumePath = false;
        newVolumePath = "";
      }
    } catch (error) {
      onMessage(`Error: ${error}`);
    }
  }

  async function openFolderPicker() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: `Select volume path for ${db.name}`,
      });

      if (selected) {
        newVolumePath = selected;
      }
    } catch (error) {
      console.error("Failed to open folder picker:", error);
      onMessage("Folder picker not available. Please enter path manually.");
    }
  }

  function getImageTag(): string {
    const parts = db.image.split(":");
    return parts.length > 1 ? parts[1] : "";
  }

  function getImageBase(): string {
    const parts = db.image.split(":");
    return parts[0] || "";
  }

  function startEditingImageTag() {
    editingImageTag = true;
    newImageTag = getImageTag();
  }

  function cancelEditingImageTag() {
    editingImageTag = false;
    newImageTag = "";
  }

  async function saveImageTag() {
    try {
      const result = await invoke<CommandResult>("set_image_tag", {
        dbName: db.name,
        tag: newImageTag,
      });
      onMessage(result.message);

      if (result.success) {
        // If tag was cleared, fetch the default from the backend
        if (!newImageTag.trim()) {
          const databases = await invoke<DatabaseInfo[]>("list_databases");
          const updatedDb = databases.find((d) => d.name === db.name);
          if (updatedDb) {
            db.image = updatedDb.image;
          }
        } else {
          db.image = `${getImageBase()}:${newImageTag}`;
        }
        editingImageTag = false;
        newImageTag = "";
      }
    } catch (error) {
      onMessage(`Error: ${error}`);
    }
  }

  $effect(() => {
    if (db.status !== "running" && showConnectionString) {
      showConnectionString = false;
    }
  });
</script>

<div
  class="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-md hover:shadow-lg transition-shadow"
>
  <div
    class="flex justify-between items-center mb-4 pb-4 border-b-2 border-gray-200 dark:border-gray-700"
  >
    <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
      {db.name}
    </h2>
    <span
      class={`px-3 py-2 rounded-full text-sm font-semibold uppercase ${
        db.status === "running"
          ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
          : "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
      }`}
    >
      {db.status}
    </span>
  </div>

  <div class="mb-6">
    <div class="flex justify-between py-2 text-gray-700 dark:text-gray-300">
      <span class="font-semibold">Image:</span>
      {#if !editingImageTag}
        <div class="flex items-center gap-2">
          <span class="font-mono text-gray-600 dark:text-gray-400">{db.image}</span>
          <button
            onclick={startEditingImageTag}
            disabled={db.status === "running" || loading}
            class={`opacity-70 hover:opacity-100 transition-opacity p-1 ${
              db.status === "running" || loading ? "cursor-not-allowed opacity-30" : ""
            }`}
            title={db.status === "running"
              ? "Stop database to edit image tag"
              : loading
                ? "Wait for operation to complete"
                : "Edit image tag"}
          >
            ✏️
          </button>
        </div>
      {:else}
        <div class="flex flex-col gap-2 flex-1 ml-4">
          <div class="flex items-center gap-2">
            <span class="font-mono text-sm text-gray-500 dark:text-gray-400">
              {getImageBase()}:
            </span>
            <input
              type="text"
              bind:value={newImageTag}
              placeholder="e.g., 16-alpine (clear to reset to default)"
              class="flex-1 px-3 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div class="flex gap-2 justify-end">
            <button
              onclick={saveImageTag}
              class="px-3 py-1 bg-green-500 text-white rounded font-semibold text-sm hover:bg-green-600 transition-colors"
            >
              Save
            </button>
            <button
              onclick={cancelEditingImageTag}
              class="px-3 py-1 bg-gray-500 text-white rounded font-semibold text-sm hover:bg-gray-600 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      {/if}
    </div>
    <div class="flex justify-between py-2 text-gray-700 dark:text-gray-300">
      <span class="font-semibold">Port:</span>
      <span class="font-mono text-gray-600 dark:text-gray-400">{db.port}</span>
    </div>

    {#if !editingVolumePath}
      <div class="flex justify-between items-center py-2 text-gray-700 dark:text-gray-300">
        <span class="font-semibold">Volume:</span>
        <div class="flex items-center gap-2">
          <span
            class="font-mono text-gray-600 dark:text-gray-400 max-w-[180px] truncate"
          >
            {db.volume_path || "Default (Docker volume)"}
          </span>
          <button
            onclick={startEditingPath}
            disabled={loading}
            class={`opacity-70 hover:opacity-100 transition-opacity p-1 ${
              loading ? "cursor-not-allowed opacity-30" : ""
            }`}
            title={loading ? "Wait for operation to complete" : "Edit volume path"}
          >
            ✏️
          </button>
        </div>
      </div>
    {:else}
      <div class="mt-2 pt-2 border-t border-gray-200 dark:border-gray-700">
        <div class="flex gap-2 mb-2">
          <input
            type="text"
            bind:value={newVolumePath}
            placeholder="Enter custom volume path..."
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onclick={openFolderPicker}
            class="px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded text-xl hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
            title="Browse for folder"
          >
            📁
          </button>
        </div>
        <div class="flex gap-2">
          <button
            onclick={saveVolumePath}
            class="flex-1 px-3 py-2 bg-green-500 text-white rounded font-semibold text-sm hover:bg-green-600 transition-colors"
          >
            Save
          </button>
          <button
            onclick={cancelEditingPath}
            class="flex-1 px-3 py-2 bg-gray-500 text-white rounded font-semibold text-sm hover:bg-gray-600 transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    {/if}

    {#if db.status === "running"}
      <ConnectionString {db} bind:isExpanded={showConnectionString} {onCopy} />
    {/if}
  </div>

  <div class="flex gap-3">
    {#if db.status === "running"}
      <button
        onclick={stopDatabase}
        disabled={loading}
        class="flex-1 px-6 py-3 bg-red-500 text-white rounded-lg font-semibold text-base hover:bg-red-600 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
      >
        {#if loading}
          <span class="flex items-center justify-center gap-2">
            <svg
              class="animate-spin h-5 w-5"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            Stopping...
          </span>
        {:else}
          ⏹ Stop
        {/if}
      </button>
      <button
        onclick={() => onOpenLogs(db)}
        class="flex-1 px-6 py-3 bg-orange-500 text-white rounded-lg font-semibold text-base hover:bg-orange-600 transition-colors"
      >
        📋 Logs
      </button>
    {:else}
      <button
        onclick={startDatabase}
        disabled={loading}
        class="flex-1 px-6 py-3 bg-blue-500 text-white rounded-lg font-semibold text-base hover:bg-blue-600 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
      >
        {#if loading}
          <span class="flex items-center justify-center gap-2">
            <svg
              class="animate-spin h-5 w-5"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            Starting...
          </span>
        {:else}
          ▶ Start
        {/if}
      </button>
    {/if}
  </div>
</div>
