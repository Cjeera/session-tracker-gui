<script lang="ts">
  import { Button } from "#lib/components/ui/button/index.js";
  import { Input } from "#lib/components/ui/input/index.js";
  import { Label } from "#lib/components/ui/label/index.js";
  import { Textarea } from "#lib/components/ui/textarea/index.js";
  import * as Table from "#lib/components/ui/table/index.js";
  import { Play, Pause, Square, X } from "@lucide/svelte";

  import { tracker } from "./sessionTracker.svelte.js";

  import { onDestroy } from 'svelte';

  onDestroy(() => {
		tracker.errorMsg = "";
    tracker.successMsg = "";
	});

</script>

<main class="mx-auto flex min-h-[calc(100vh-5rem)] max-w-3xl flex-col items-center justify-center gap-8 px-4 py-12">
  <form class="flex w-full max-w-md flex-col items-center" onsubmit={tracker.searchProcesses}>
    <h2 class="mb-8 text-center text-4xl font-bold">{tracker.headerMessage}</h2>
    {#if tracker.gameFound}
      <h3 class="mb-4 font-mono text-2xl font-bold tabular-nums" role="timer">{tracker.stopwatchDisplay}</h3>
      <div class="flex h-8 items-center justify-center">
        {#if tracker.paused && Object.keys(tracker.sessionData).length === 0}
          <h4 class="text-xl font-semibold">Session Paused!</h4>
        {/if}
      </div>
      {#if Object.keys(tracker.sessionData).length === 0}
        <div class="mt-4 flex gap-2">
          {#if tracker.paused}
            <Button type="button" variant="outline" onclick={tracker.resumeSession}><Play /> Resume</Button>
          {:else}
            <Button type="button" variant="outline" onclick={tracker.pauseSession}><Pause /> Pause</Button>
          {/if}
          <Button type="button" variant="outline" onclick={tracker.userStopSession}><Square /> Stop</Button>
        </div>
      {/if}
    {:else}
      <div class="w-full space-y-2">
        <Label for="game-input">Game or process name</Label>
        <div class="flex gap-2">
          <Input id="game-input" bind:value={tracker.gameInput} placeholder="Enter a name..." required aria-invalid={tracker.errorFlag} aria-describedby={tracker.errorFlag ? "search-error" : undefined} />
          <Button type="button" variant="outline" size="icon" aria-label="Clear game name" onclick={() => tracker.gameInput = ""}><X /></Button>
        </div>
        {#if tracker.errorFlag}
          <p id="search-error" class="text-sm text-destructive" role="alert">{tracker.errorMsg}</p>
        {/if}
      </div>
      <Button type="submit" class="mt-4">Enter</Button>
      {#if tracker.successMsg.length > 0}
        <p class="mt-3 text-sm" role="status">{tracker.successMsg}</p>
      {/if}
      {#if tracker.searchSuccessful}
        <div class="mt-6 w-full rounded-lg border">
          <Table.Root>
            <Table.Caption>Results — choose a process to track</Table.Caption>
            <Table.Header><Table.Row><Table.Head>PID</Table.Head><Table.Head>Name</Table.Head></Table.Row></Table.Header>
            <Table.Body>
              {#each tracker.searchResults as process}
                <Table.Row class="cursor-pointer active:scale-98 transition-all" onclick={() => tracker.trackSession({ pid: process.pid, name: process.name })}>
                  <Table.Cell>{process.pid}</Table.Cell>
                  <Table.Cell><Button type="button" variant="ghost" class="h-auto justify-start whitespace-normal text-left">{process.name}</Button></Table.Cell>
                </Table.Row>
              {/each}
            </Table.Body>
          </Table.Root>
        </div>
      {/if}
    {/if}
  </form>

  {#if Object.keys(tracker.sessionData).length > 0}
    <form class="flex w-full max-w-md flex-col gap-3" onsubmit={tracker.endSession}>
      <Label for="new-game-input">New title (optional)</Label>
      <div class="flex gap-2">
        <Input id="new-game-input" bind:value={tracker.newGameInput} placeholder="Enter a new title..." />
        <Button type="button" variant="outline" size="icon" aria-label="Clear new title" onclick={() => tracker.newGameInput = ""}><X /></Button>
      </div>
      <Label for="notes-input">Session notes (optional)</Label>
      <Textarea id="notes-input" class="min-h-24" placeholder="Enter session notes..." bind:value={tracker.sessionNotes} />
      <Button type="submit" class="self-center">Enter</Button>
      <p class="text-center text-sm text-muted-foreground">Click Enter to save, even if you leave the title and notes empty.</p>
    </form>
  {/if}
</main>
