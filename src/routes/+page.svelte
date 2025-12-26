<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import DatabaseCard from "$lib/components/DatabaseCard.svelte";
  import LogsModal from "$lib/components/LogsModal.svelte";

  interface DatabaseInfo {
    name: string;
    status: string;
    port: number;
    image: string;
    volume_path?: string;
  }

  type Theme = "light" | "dark";

  let databases = $state<DatabaseInfo[]>([]);
  let message = $state("");
  let theme = $state<Theme>("light");
  let showingLogs = $state<string | null>(null);

  onMount(async () => {
    const savedTheme = localStorage.getItem("theme") as Theme | null;
    if (savedTheme) {
      theme = savedTheme;
    } else {
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;
      theme = prefersDark ? "dark" : "light";
    }
    applyTheme();

    await loadDatabases();
  });

  function applyTheme() {
    if (typeof document !== "undefined") {
      if (theme === "dark") {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }
    }
  }

  $effect(() => {
    localStorage.setItem("theme", theme);
    applyTheme();
  });

  async function loadDatabases() {
    databases = await invoke<DatabaseInfo[]>("list_databases");
    for (const db of databases) {
      await updateDatabaseStatus(db);
    }
  }

  async function updateDatabaseStatus(db: DatabaseInfo) {
    const status = await invoke<string>("get_database_status", {
      dbName: db.name,
    });
    db.status = status;
  }

  async function refreshStatuses() {
    message = "Refreshing database statuses...";
    for (const db of databases) {
      await updateDatabaseStatus(db);
    }
    message = "Statuses refreshed";
  }

  function handleMessage(msg: string) {
    message = msg;
  }

  function dismissMessage() {
    message = "";
  }

  function handleOpenLogs(db: DatabaseInfo) {
    showingLogs = db.name;
  }

  function handleCloseLogs() {
    showingLogs = null;
  }

  async function handleCopy(text: string, dbName: string) {
    try {
      await navigator.clipboard.writeText(text);
      message = `Copied connection string for ${dbName}`;
      setTimeout(() => {
        if (message.startsWith("Copied connection string")) {
          message = "";
        }
      }, 2000);
    } catch (error) {
      message = `Failed to copy: ${error}`;
    }
  }
</script>

<main class="min-h-screen bg-gray-100 dark:bg-gray-900 transition-colors">
  <div class="max-w-7xl mx-auto px-8 py-8">
    <div class="flex justify-between items-start mb-8">
      <div class="w-12"></div>
      <div class="flex-1 flex flex-col items-center">
        <h1 class="text-4xl font-bold text-gray-900 dark:text-gray-100">
          DB Runner
        </h1>
        <p class="mt-2 text-gray-600 dark:text-gray-400">
          Manage your development databases with ease
        </p>
      </div>
      <ThemeToggle bind:theme />
    </div>

    <div class="flex justify-end mb-6">
      <button
        onclick={refreshStatuses}
        class="px-6 py-3 bg-green-500 text-white rounded-lg font-semibold text-base hover:bg-green-600 transition-colors"
      >
        🔄 Refresh Status
      </button>
    </div>

    {#if message}
      <div
        class="bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 px-4 py-4 rounded-lg mb-6 border-l-4 border-blue-500 transition-all flex items-center justify-between gap-4"
      >
        <span class="flex-1 text-center">{message}</span>
        <button
          onclick={dismissMessage}
          class="flex-shrink-0 text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-200 transition-colors p-1 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50"
          aria-label="Dismiss message"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fill-rule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
      </div>
    {/if}

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each databases as db, i (db.name)}
        <DatabaseCard
          bind:db={databases[i]}
          onMessage={handleMessage}
          onOpenLogs={handleOpenLogs}
          onCopy={handleCopy}
        />
      {/each}
    </div>
  </div>
</main>

{#if showingLogs}
  <LogsModal dbName={showingLogs} onClose={handleCloseLogs} />
{/if}
