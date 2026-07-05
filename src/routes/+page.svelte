<script lang="ts">
  import "../app.css"

  import {
    FloatingLabelInput,
    Textarea,
    Button,
    Helper,
  } from "flowbite-svelte";

  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from "flowbite-svelte";

  import PlaySolid from "flowbite-svelte-icons/PlaySolid.svelte";
  import PauseSolid from "flowbite-svelte-icons/PauseSolid.svelte";
  import StopSolid from "flowbite-svelte-icons/StopSolid.svelte";

  import { tracker } from "./sessionTracker.svelte.js";

  import { onDestroy } from 'svelte';

  onDestroy(() => {
		tracker.errorMsg = "";
    tracker.successMsg = "";
	});

</script>

<main
  class="min-h-screen flex flex-col justify-center items-center gap-8 text-white"
>
  <!--STOPWATCH DISPLAY-->
  <form class="flex flex-col items-center" onsubmit={tracker.searchProcesses}>
    <h2 class="text-4xl font-bold mb-8 text-center">{tracker.headerMessage}</h2>
    <!--Displays stopwatch if gameFound is true-->
    {#if tracker.gameFound}
      <h3 class="text-2xl font-bold mb-4">{tracker.stopwatchDisplay}</h3>

        <div class="h-8 flex items-center justify-center">
          {#if tracker.paused && Object.keys(tracker.sessionData).length == 0}
              <h4 class="text-2xl font-bold">Session Paused!</h4>
          {/if}
        </div>

        <div class="grid grid-cols-2 gap-2 items-center justify-center">

          <!--Displays resume button if tracker is paused-->
          {#if tracker.paused && Object.keys(tracker.sessionData).length == 0}
            <Button onclick={tracker.resumeSession} outline color="blue" class="w-24 mt-4 cursor-pointer transition-colors">
              Resume
              <PlaySolid/>
            </Button>

          <!--Displays paused button if tracker is running-->
          {:else if !tracker.paused && Object.keys(tracker.sessionData).length == 0}
            <Button onclick={tracker.pauseSession} outline color="blue" class="w-24 mt-4 cursor-pointer transition-colors">
              Pause
              <PauseSolid/>
            </Button>
          {/if}

          <!--Stop session button-->
          {#if Object.keys(tracker.sessionData).length == 0}
            <Button onclick={tracker.userStopSession} outline color="blue" class="w-24 mt-4 cursor-pointer transition-colors">
              Stop
            <StopSolid/>
            </Button>
          {/if}

        </div>
    <!--GAME SEARCH SECTION-->
    {:else if !tracker.gameFound}
      <FloatingLabelInput
        class="w-96"
        color="blue"
        clearable
        clearableColor="blue"
        variant="outlined"
        id="game-input"
        bind:value={tracker.gameInput}
        type="text"
        classes={{ label: "!bg-primary", close:"cursor-pointer transition-colors" }}
        required
      >
        Enter a name...
      </FloatingLabelInput>
      <div class="min-h-4 items-end">
        {#if tracker.errorFlag}
          <Helper color="red">
            <span class="font-medium text-left">{tracker.errorMsg}</span>
          </Helper>
        {/if}
      </div>
      <Button type="submit" outline color="blue" class="mt-4 cursor-pointer transition-colors"
        >Enter</Button
      >
      {#if tracker.successMsg.length > 0}
        <p>{tracker.successMsg}</p>
      {/if}
      <!--Displays a table of search results if there are any results-->
      {#if tracker.searchSuccessful}
        <Table hoverable={true} class="mt-4 w-96">
        <caption class="p-2 text-left text-lg font-semibold">Results</caption>
          <TableHead color="primary">
            <TableHeadCell>PID</TableHeadCell>
            <TableHeadCell>Name</TableHeadCell>
          </TableHead>
          <TableBody class="cursor-pointer">
            {#each tracker.searchResults as process}
            <!--trackSession function is called when a search result is clicked-->
              <TableBodyRow onclick={() => tracker.trackSession({pid: process.pid, name: process.name})} class="bg-primary! border-blue-500! hover:bg-gray-800! cursor-pointer transition-colors">
                <TableBodyCell>{process.pid}</TableBodyCell>
                <TableBodyCell>{process.name}</TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      {/if}
    {/if}
  </form>

  <!--ALT NAME AND NOTES SECTION-->
  {#if Object.keys(tracker.sessionData).length > 0}
    <form class="flex flex-col items-center" onsubmit={tracker.endSession}>
      <FloatingLabelInput
        class="w-96 mb-3"
        color="blue"
        clearable
        clearableColor="blue"
        variant="outlined"
        id="new-game-input"
        bind:value={tracker.newGameInput}
        type="text"
        classes={{ label: "!bg-primary" }}
      >
        Enter a new title... (Optional)
      </FloatingLabelInput>
      <Textarea
        id="notes-input"
        class="p-2 rounded mb-2 mt-3 w-96 h-24 placeholder-blue-400!"
        classes={{ wrapper: "!bg-primary" }}
        placeholder="Enter session notes... (Optional)"
        bind:value={tracker.sessionNotes}
      />
      <Button type="submit" outline color="blue" class="mt-4 cursor-pointer"
        >Enter</Button
      >
      <p>
        If you don't wish to enter any notes or overwrite entered title, click enter
      </p>
    </form>
  {/if}
</main>
