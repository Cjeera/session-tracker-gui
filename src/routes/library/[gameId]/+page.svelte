<script lang="ts">
    import * as Tabs from "#lib/components/ui/tabs/index.js";
    import * as Dialog from "#lib/components/ui/dialog/index.js";
    import * as RadioGroup from "#lib/components/ui/radio-group/index.js";
    import { Label } from "#lib/components/ui/label/index.js";
    import { Button } from "#lib/components/ui/button/index.js";
    import { invoke } from "@tauri-apps/api/core";
    import { page } from "$app/state";
    
    // Component imports
    import SessionList from "./SessionList.svelte";
    import SessionTimeline from "./SessionTimeline.svelte";
    import Stats from "./Stats.svelte";
    import { formatDuration, formatLocaleDate, formatTime } from "#lib/timeFormatting.js";
    import type { GameCover, Game, GameStats, Session, GameTimeRangeStats } from "#lib/types.js";

    interface RouteParams {
        gameId: string;
    }

    // The game id passed from the library page.
    let params = $derived(page.params as unknown as RouteParams);
    let rawId = $derived(params.gameId);

    let modalState = $state(false);
    
    // Data stores for the fetched backend information
    let game = $state<Partial<Game>>({});
    let gameStats = $state<Partial<GameStats>>({});
    let gameWeeklyStats = $state<Partial<GameTimeRangeStats[]>>([]);
    let gameMonthlyStats = $state<Partial<GameTimeRangeStats[]>>([]);
    let gameYearlyStats = $state<Partial<GameTimeRangeStats[]>>([]);
    let sessions = $state<Session[]>([]);
    let covers = $state<GameCover[]>([]);

    // UI state trackers
    let errorMsg = $state("");
    let radioGroup = $state("");


    /** * Fetches all tracked sessions for the currently selected game.
     */
    async function getSessions() {
        // Clear the array before fetching to prevent stale data
        sessions = [];
        
        try {
            // Convert the URL string ID into a number for Rust's u32/i32 expectation
            let numericId = Number(rawId);
            
            // Await the response from the Rust backend
            sessions = await invoke("get_game_sessions", { gameId: numericId });
        } catch (error) {
            console.error("Failed to load sessions:", error);
        }
    }

    /** * Fetches the core metadata (title, cover path, etc.) for a single game.
     */
    async function getGame() {
        try {
            // Convert the URL string ID into a number
            let numericId = Number(rawId);
            
            // Await the game data from the Rust backend
            game = await invoke("get_single_game", { gameId: numericId });
            radioGroup = String(game.status);
        } catch (error) {
            errorMsg = String(error);
            console.error(error);
        }
    }

    /** Fetches aggregated statistics (total playtime, session counts) for the game.
     */
    async function getGameStats() {
        // Reset state before fetching
        errorMsg = "";
        gameStats = {};
        gameWeeklyStats = [];
        gameMonthlyStats = [];
        gameYearlyStats = [];

        try {
            // Convert the URL string ID into a number
            let numericId = Number(rawId);
            
            // Await the stats payload from the Rust backend
            gameStats = await invoke("get_game_stats", { gameId: numericId });      
            gameWeeklyStats = await invoke("get_game_weekly_stats", { gameId: numericId });
            gameMonthlyStats = await invoke("get_game_monthly_stats", { gameId: numericId });
            gameYearlyStats = await invoke("get_game_yearly_stats", { gameId: numericId });
            
        } catch (error) {
            errorMsg = String(error);
            console.error("Failed to load stats:", error);
        }
    }

    /**Calls backend function to get list of alternate cover art*/
    async function getAltCovers(title: string, isAutoFetch: boolean) {
        try {
            covers = await invoke("fetch_cover_art", {name: title, isAutoFetch})
        } catch(error) {
            errorMsg = String(error);
            console.error(error);
        }
    }

    /**Calls backend function to insert a new game cover*/
    async function insertNewCover(cover: GameCover, gameId: number, isAutoFetch: boolean) {
        try {
            await invoke("insert_selected_cover", {cover: cover, gameId: gameId, isAutoFetch: isAutoFetch})
            game.coverPath = cover.cover?.url;
            modalState = false;
        } catch(error) {
            errorMsg = String(error);
            console.error(error);
        }
    }

    /**Calls backend function to update status of game*/
    async function updateStatus(status: string) {
        if (status == game.status) {
            return;
        }

        try {
            let numericId = Number(rawId)
            await invoke("update_game_status", {gameId: numericId, status: status})
            game.status = status;
        } catch (error) {
            radioGroup = String(game.status);
            errorMsg = String(error);
            console.error(error);
        }
    }

    // Automatically trigger data fetches when the component is mounted to the DOM.
    $effect(() => {
        // Only attempt to fetch data if an ID was successfully parsed from the URL.
        if (rawId) {
            getGame();
            getGameStats();
            getSessions();
        }
    });
</script>

<main class="min-h-screen p-8">
    <div class="flex flex-col md:flex-row items-start gap-12 max-w-7xl mx-auto">

        <!--GAME INFO DISPLAY SECTION-->
        <div class="flex flex-col items-center w-full md:w-1/3 shrink-0 text-foreground">
            
            <img
                src={game.coverPath || "/placeholder.avif"}
                alt="{game.title} Cover Art"
                class="w-64 h-96 object-cover rounded-lg border"
            />

            <Button variant="link" onclick={() => { modalState = true; getAltCovers(String(game.title), false); }}>Change Cover Art</Button>
            <Dialog.Root bind:open={modalState}>
                <Dialog.Content class="max-h-[85vh] overflow-y-auto sm:max-w-3xl">
                    <Dialog.Header>
                        <Dialog.Title>Change Cover Art</Dialog.Title>
                        <Dialog.Description>Choose the cover art you wish to use.</Dialog.Description>
                    </Dialog.Header>
                    {#if errorMsg}<p class="text-destructive" role="alert">{errorMsg}</p>{/if}
                    <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
                        {#each covers as cover}
                            <button class="hover:opacity-70 cursor-pointer active:translate-y-2 transition-all overflow-hidden rounded-md border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-label="Use this cover for {game.title}" onclick={() => insertNewCover(cover, Number(game.gameId), false)}>
                                <img src={cover.cover?.url} alt="{game.title} alternate cover" class="aspect-3/4 w-full object-cover" />
                            </button>
                        {/each}
                    </div>
                </Dialog.Content>
            </Dialog.Root>

            {#if errorMsg}
                <div class="text-foreground p-4 rounded mb-4 font-bold">
                Error: {errorMsg}
                </div>
            {/if}

            <h1 class="text-4xl font-bold mt-2 text-center">
                {game.title}
            </h1>

            <div class="mt-4 flex flex-col items-center text-xl gap-2 font-semibold">
                <p>Total Hours Played: {gameStats.totalPlaytime ? formatDuration(gameStats.totalPlaytime) : "00:00:00"}</p>
                <p>Average Session Length: {gameStats.averageSessionLength ? formatDuration(gameStats.averageSessionLength) : "00:00:00"}</p>
                <p>Total Sessions: {gameStats.totalSessions || 0}</p>
                <p>Average Start Time: {gameStats.averageStartTime ?? "Not Played"}</p>
                <p>Average End Time: {gameStats.averageEndTime ?? "Not Played"}</p>
                <p>Last Played: {gameStats.lastPlayed ? formatLocaleDate(gameStats.lastPlayed) : "Not Played"}</p>

                <RadioGroup.Root value={radioGroup} onValueChange={(value) => { radioGroup = value; updateStatus(value); }} class="flex flex-wrap gap-4" aria-label="Game status">
                    {#each ["played", "playing", "backlog"] as status}
                        <div class="flex items-center gap-2">
                            <RadioGroup.Item id={"status-" + status} value={status} class="cursor-pointer" />
                            <Label for={"status-" + status} class="cursor-pointer capitalize">{status}</Label>
                        </div>
                    {/each}
                </RadioGroup.Root>
            </div>
        </div>

        <!--TABS BAR SECTION-->
        <div class="w-full md:w-2/3 overflow-x-auto">
            <Tabs.Root value="sessions">
                <Tabs.List class="mb-4">
                    <Tabs.Trigger value="sessions">Sessions</Tabs.Trigger>
                    <Tabs.Trigger value="timeline">Timeline</Tabs.Trigger>
                    <Tabs.Trigger value="stats">Stats</Tabs.Trigger>
                </Tabs.List>
                <Tabs.Content value="sessions"><SessionList bind:sessions /></Tabs.Content>
                <Tabs.Content value="timeline"><SessionTimeline {sessions} /></Tabs.Content>
                <Tabs.Content value="stats"><Stats {sessions} /></Tabs.Content>
            </Tabs.Root>
        </div>
    </div>
</main>
