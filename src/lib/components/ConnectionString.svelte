<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface DatabaseInfo {
    name: string;
    status: string;
    port: number;
    image: string;
    volume_path?: string;
  }

  interface ConnectionStringData {
    standardUri: string;
    jdbc: string;
    components: {
      host: string;
      port: number;
      user: string;
      password: string;
      database: string;
    };
  }

  let {
    db,
    isExpanded = $bindable(false),
    onCopy,
  }: {
    db: DatabaseInfo;
    isExpanded?: boolean;
    onCopy: (text: string, dbName: string) => void;
  } = $props();

  let connectionStrings = $state<ConnectionStringData | null>(null);
  let selectedFormat = $state<string>("standard_uri");
  let copiedRecently = $state(false);

  async function loadConnectionStrings() {
    try {
      const result = await invoke<Record<string, string>>(
        "generate_connection_strings",
        { dbName: db.name, port: db.port },
      );

      connectionStrings = {
        standardUri: result.standard_uri,
        jdbc: result.jdbc,
        components: {
          host: result.host,
          port: parseInt(result.port),
          user: result.user,
          password: result.password,
          database: result.database,
        },
      };
    } catch (error) {
      console.error("Error loading connection strings:", error);
    }
  }

  function toggleExpanded() {
    isExpanded = !isExpanded;
    if (isExpanded && !connectionStrings) {
      loadConnectionStrings();
    }
  }

  function getCurrentConnectionString(): string {
    if (!connectionStrings) return "";

    if (selectedFormat === "standard_uri") {
      return connectionStrings.standardUri;
    } else if (selectedFormat === "jdbc") {
      return connectionStrings.jdbc;
    } else {
      return JSON.stringify(connectionStrings.components, null, 2);
    }
  }

  async function handleCopy() {
    const text =
      selectedFormat === "components"
        ? JSON.stringify(connectionStrings!.components, null, 2)
        : getCurrentConnectionString();

    onCopy(text, db.name);
    copiedRecently = true;
    setTimeout(() => {
      copiedRecently = false;
    }, 2000);
  }
</script>

<div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
  <button
    onclick={toggleExpanded}
    class="flex items-center gap-2 font-semibold text-gray-900 dark:text-gray-100 text-base px-2 py-2 -mx-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-all w-full"
    aria-expanded={isExpanded}
  >
    <span class="text-sm transition-transform inline-flex items-center">
      {isExpanded ? "▼" : "▶"}
    </span>
    Connection String
  </button>

  {#if isExpanded}
    <div
      class="mt-3 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700"
    >
      {#if connectionStrings}
        <div class="flex items-center gap-3 mb-4">
          <label for="format-select" class="font-semibold text-gray-900 dark:text-white text-sm">
            Format:
          </label>
          <select
            id="format-select"
            bind:value={selectedFormat}
            class="flex-1 px-4 py-2.5 border-2 border-gray-800 dark:border-gray-400 rounded-lg bg-white dark:bg-gray-800 text-sm font-semibold cursor-pointer hover:border-blue-600 dark:hover:border-blue-400 focus:outline-none focus:border-blue-600 focus:ring-4 focus:ring-blue-500/30 transition-all text-gray-900 dark:text-white"
            style="color: rgb(17, 24, 39); appearance: auto;"
          >
            <option value="standard_uri" style="color: rgb(17, 24, 39);" class="text-gray-900 dark:text-white font-semibold">Standard URI</option>
            <option value="jdbc" style="color: rgb(17, 24, 39);" class="text-gray-900 dark:text-white font-semibold">JDBC Format</option>
            <option value="components" style="color: rgb(17, 24, 39);" class="text-gray-900 dark:text-white font-semibold">Components</option>
          </select>
        </div>

        <div class="flex flex-col gap-3">
          {#if selectedFormat === "components"}
            <div
              class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded p-3"
            >
              {#each [
                ["Host", connectionStrings.components.host],
                ["Port", connectionStrings.components.port],
                ["User", connectionStrings.components.user],
                ["Password", connectionStrings.components.password],
                ["Database", connectionStrings.components.database],
              ] as [label, value]}
                <div
                  class="flex justify-between py-2 text-sm border-b border-gray-200 dark:border-gray-700 last:border-b-0"
                >
                  <span class="font-semibold text-gray-700 dark:text-gray-300 min-w-[80px]">
                    {label}:
                  </span>
                  <span
                    class="font-mono text-gray-600 dark:text-gray-400 text-right flex-1 break-all"
                  >
                    {value}
                  </span>
                </div>
              {/each}
            </div>
          {:else}
            <div
              class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded p-3 font-mono text-sm text-gray-600 dark:text-gray-400 break-all leading-relaxed"
            >
              {getCurrentConnectionString()}
            </div>
          {/if}

          <button
            onclick={handleCopy}
            class="self-start px-4 py-2 bg-blue-500 text-white rounded-md font-semibold text-sm hover:bg-blue-600 hover:-translate-y-0.5 active:translate-y-0 transition-all"
          >
            {copiedRecently ? "Copied!" : "📋 Copy"}
          </button>
        </div>
      {:else}
        <div class="text-center py-4 text-gray-500 dark:text-gray-400 italic">
          Loading connection strings...
        </div>
      {/if}
    </div>
  {/if}
</div>
