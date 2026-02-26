<script>
  import { invoke } from "@tauri-apps/api/core";
  import "../app.css";
  import { Button, Input } from "flowbite-svelte";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<div class="min-h-screen text-white">
  <main class="container mx-auto py-10 text-center">
    <h1 class="text-2xl font-bold mb-4">Welcome to Tauri + Svelte</h1>

    <form class="flex flex-col items-center gap-4" onsubmit={greet}>
      <Input 
        id="greet-input"
        class="p-2 rounded mb-2 max-w-2xs" 
        placeholder="Enter a name..." 
        bind:value={name} 
      />
      <Button type="submit" outline color="blue" class="text-white cursor-pointer">Greet</Button>
    </form>
    
    <p class="mt-4">{greetMsg}</p>
  </main>
</div>
