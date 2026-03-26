<script lang="ts">
    import { Tabs, TabItem } from "flowbite-svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { page } from "$app/state";
    
    // Component imports
    import SessionList from "./SessionList.svelte";
    import SessionTimeline from "./SessionTimeline.svelte";
    import Stats from "./Stats.svelte";
    import { formatDuration } from "./timeFormatting";
    import type { Game, GameStats, Session } from "./types";

    // Reactively extracts the 'id' parameter from the current URL string.
    let rawId = $derived(page.url.searchParams.get("id"));
    
    // Data stores for the fetched backend information
    let game = $state<Partial<Game>>({});
    let gameStats = $state<Partial<GameStats>>({});
    let sessions = $state<Session[]>([]);

    // UI state trackers
    let errorMsg = $state();
    let success = $state(false);

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
        } catch (error) {
            errorMsg = error;
            console.error("Failed to load game", error);
        }
    }

    /** * Fetches aggregated statistics (total playtime, session counts) for the game.
     */
    async function getGameStats() {
        // Reset state before fetching
        errorMsg = "";
        success = false;
        gameStats = {};

        try {
            // Convert the URL string ID into a number
            let numericId = Number(rawId);
            
            // Await the stats payload from the Rust backend
            gameStats = await invoke("get_game_stats", { gameId: numericId });
            
            // Mark the fetch as successful to update dependent UI elements
            success = true;
        } catch (error) {
            errorMsg = error;
            console.error("Failed to load stats:", error);
        }
    }

    // Automatically trigger data fetches when the component is mounted to the DOM.
    onMount(() => {
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

        <div class="flex flex-col items-center w-full md:w-1/3 shrink-0 text-white">
            
            <img
                src={game.coverPath || "../placeholder.avif"}
                alt="game cover art"
                class="w-64 h-96 object-cover border-2 border-blue-500"
            />

            <h1 class="text-4xl font-bold mt-6 text-center">
                {game.title}
            </h1>

            <div class="mt-4 flex flex-col items-center text-xl gap-2 font-semibold">
                <p>Total Hours Played: {gameStats.totalPlaytime ? formatDuration(gameStats.totalPlaytime) : "00:00:00"}</p>
                <p>Total Sessions: {gameStats.totalSessions || 0}</p>
            </div>

        </div>

        <div class="w-full md:w-2/3 overflow-x-auto text-blue-500">
            <Tabs tabStyle="underline" classes={{ content:"bg-primary!", divider: "bg-blue-500!"} }>
                
                <TabItem title="Sessions" classes={{ button:"cursor-pointer"}}> 
                    <SessionList {sessions}/>
                </TabItem>
                
                <TabItem title="Timeline" classes={{ button:"cursor-pointer"}}> 
                    <SessionTimeline {sessions}/>
                </TabItem>
                
                <TabItem title="Stats" classes={{ button:"cursor-pointer"}}>
                    <Stats {sessions}/>
                </TabItem>
                
            </Tabs>
        </div>

    </div>
</main>