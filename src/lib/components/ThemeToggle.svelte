<script lang="ts">
  type Theme = "light" | "dark";

  let { theme = $bindable<Theme>("light") }: { theme: Theme } = $props();

  function toggleTheme() {
    theme = theme === "light" ? "dark" : "light";
  }
</script>

<button
  onclick={toggleTheme}
  class="flex items-center justify-center w-12 h-12 bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 rounded-full text-2xl shadow-md hover:scale-110 hover:shadow-lg transition-all duration-300"
  aria-label="Toggle theme"
>
  {theme === "light" ? "🌙" : "☀️"}
</button>
