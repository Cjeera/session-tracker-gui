<script>
  import { invoke } from "@tauri-apps/api/core";
  import "../app.css";
  import { Button, Input } from "flowbite-svelte";

  let gameInput = $state("");
  let sessionNotes = $state("");
  let resultMsg = $state("");
  let success = $state(false);
  let sessionData = $state({});
  let pid = $state(0);

  async function trackSession(event) {
    event.preventDefault();
    success = false;
    resultMsg = "";

    if (gameInput.length <= 2) {
      resultMsg = "Enter a name at least 3 characters long";
      return;
    }

    pid = await invoke("search_processes", { gameInput } ).catch (
      (error) => {
        resultMsg = error;
        console.error(error);
        return;
      }
    )

    if (!pid) {
      return;
    }

    resultMsg = "Found " + gameInput + " (PID " + pid + ")";

    const result = await invoke("start_tracker", { gameInput, pid }).catch(
      (error) => {
        resultMsg = error;
        console.error(error);
      },
    );
    
    if (!result) {
      return;
    }

    success = true;

    sessionData = result;

    console.log(sessionData);
  }

  function endSession(event) {
    event.preventDefault();
    invoke("end_tracker", { sessionNotes, sessionData }).catch(
      (error) => {
        console.error(error);
      },
    );
  }
</script>

<main class="flex flex-col p-64 items-center text-center text-white">
  <form class="flex flex-col items-center gap-4" onsubmit={trackSession}>
    <h2 class="text-4xl font-bold mb-4">Enter a game title to get started</h2>
    <Input
      id="game-input"
      class="p-2 rounded mb-2 max-w-xs"
      placeholder="Enter a name..."
      bind:value={gameInput}
      required
    />
    <Button type="submit" outline color="blue" class="cursor-pointer"
      >Enter</Button
    >
    <p>{resultMsg}</p>
  </form>

  {#if success == true}
    <form class="flex flex-col items-center gap-4" onsubmit={endSession}>
      <h2 class="text-4xl font-bold mb-4">Enter session notes</h2>
      <Input
        id="notes-input"
        class="p-2 rounded mb-2 max-w-xs"
        placeholder="Enter session notes..."
        bind:value={sessionNotes}
      />
      <Button type="submit" outline color="blue" class="cursor-pointer"
        >Enter</Button
      >
    </form>
  {/if}
</main>
