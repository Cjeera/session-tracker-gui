<script>
  import { invoke } from "@tauri-apps/api/core";
  import "../app.css";
  import { Button, Input } from "flowbite-svelte";
  import { listen } from "@tauri-apps/api/event";

  let headerMessage = $state("Enter a game title to get started");
  let gameInput = $state("");
  let sessionNotes = $state("");
  let resultMsg = $state("");
  let gameFound = $state(false);
  let sessionData = $state({});
  let pid = $state(0);
  let StopwatchPayload = $state({
    elapsedMs: 0,
    isRunning: false,
  });
  let seconds = $state(0);
  let minutes = $state(0);
  let hours = $state(0);
  let stopwatchDisplay = $state("Elapsed Time: ");

  async function trackSession(event) {
    event.preventDefault();

    resultMsg = "";

    // Any input less or equal to characters will be rejected.
    if (gameInput.length <= 2) {
      resultMsg = "Enter a name at least 3 characters long";
      return;
    }

    // gameInput is sent to the backend.
    pid = await invoke("search_processes", { gameInput }).catch((error) => {
      resultMsg = error;
      console.error(error);
      return;
    });

    // Function returns if no process ID is found.
    if (!pid) {
      return;
    }

    gameFound = true;

    // Header message displays confirmation message.
    headerMessage =
      "Found " + gameInput + " (PID " + pid + ")! Tracking Started";

    // The elapsed time in milliseconds is taken from backend and formatted into HH:MM:SS
    await listen("stopwatch-tick", (event) => {
      StopwatchPayload.elapsedMs = event.payload.elapsedMs;
      StopwatchPayload.isRunning = event.payload.isRunning;
      let totalSeconds = Math.floor(StopwatchPayload.elapsedMs / 1000);
      hours = Math.floor(totalSeconds / 3600);
      minutes = Math.floor(totalSeconds / 60) % 60;
      seconds = Math.floor(totalSeconds % 60);
      stopwatchDisplay =
        "Elapsed Time:\n" +
        String(hours).padStart(2, "0") +
        ":" +
        String(minutes).padStart(2, "0") +
        ":" +
        String(seconds).padStart(2, "0");
    });
    // The main tracker backend logic is called.
    const result = await invoke("start_tracker", { gameInput, pid }).catch(
      (error) => {
        resultMsg = error;
        console.error(error);
      },
    );

    if (!result) {
      return;
    }

    // The session data object returned by the backend function is assigned to sessionData.
    sessionData = result;
    headerMessage = "Sesssion ended!"
  }

  // The sessionData object and the sessionNotes string are sent to the backend for database insertion.
  function endSession(event) {
    event.preventDefault();
    invoke("end_tracker", { sessionNotes, sessionData }).catch((error) => {
      resultMsg = error;
      console.error(error);
    });

    resultMsg = "Session data saved to database!";

    // Session data, game input, session notes, header message and game found are reset to original state.
    sessionData = {};
    gameInput = "";
    sessionNotes = "";
    headerMessage = "Enter a game title to get started";
    gameFound = false;
  }
</script>

<main class="flex flex-col p-64 items-center text-center text-white">
  <form class="flex flex-col items-center gap-4" onsubmit={trackSession}>
    <h2 class="text-4xl font-bold mb-4">{headerMessage}</h2>
    {#if gameFound}
      <h3 class="text-2xl font-bold mb-4">{stopwatchDisplay}</h3>
    {/if}
    {#if !gameFound}
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
    {/if}
  </form>

  {#if Object.keys(sessionData).length > 0}
    <form class="flex flex-col items-center gap-4" onsubmit={endSession}>
      <h2 class="text-2xl font-bold mb-4">Enter session notes</h2>
      <Input
        id="notes-input"
        class="p-2 rounded mb-2 max-w-xs"
        placeholder="Enter session notes..."
        bind:value={sessionNotes}
      />
      <Button type="submit" outline color="blue" class="cursor-pointer"
        >Enter</Button
      >
      <p>If you don't wish to enter any notes, click enter</p>
    </form>
  {/if}
</main>
