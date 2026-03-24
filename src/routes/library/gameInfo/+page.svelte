<script>
    import { Card, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from "flowbite-svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { page } from "$app/state";

    let game = $state({});
    
    // Grab the ID from the URL as a string
    let rawId = $derived(page.url.searchParams.get("id"));
    
    let gameStats = $state({});
    let gameSessions = $state([]);
    let errorMsg = $state("");
    let success = $state(false);

    async function getGame() {
        try {
            // Convert to a number so Rust accepts it!
            let numericId = Number(rawId);
            game = await invoke("get_single_game", { gameId: numericId });
        } catch (error) {
            errorMsg = error;
            console.error("Failed to load game", error);
        }
    }

    async function getGameStats() {
        errorMsg = "";
        success = false;
        gameStats = {};

        try {
            // Pass the ID here!
            let numericId = Number(rawId);
            gameStats = await invoke("get_game_stats", { gameId: numericId });
            success = true;
        } catch (error) {
            errorMsg = error;
            console.error("Failed to load stats:", error);
        }
    }

    async function getSessions() {
        gameSessions = [];
        try {
            // Pass the ID here too!
            let numericId = Number(rawId);
            gameSessions = await invoke("get_game_sessions", { gameId: numericId });
            success = true;
        } catch (error) {
            errorMsg = error;
            console.error("Failed to load sessions:", error);
        }
    }

    onMount(() => {
        // Only run if we actually have an ID in the URL
        if (rawId) {
            getGame();
            getGameStats();
            getSessions();
        }
    });

    function formatDate(timestampString) {
        const date = new Date(timestampString);

        return date.toLocaleDateString();
    }

    function formatTime(timestampString) {
        const date = new Date(timestampString);

        return date.toLocaleTimeString();
    }

    function formatDuration(durationSeconds) {
        let hours = Math.floor( durationSeconds / 3600);
        let minutes = Math.floor((durationSeconds % 3600) / 60);
        let seconds = durationSeconds % 60;

        ;

        return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
    }
</script>

<main class="min-h-screen text-white p-8">
    
    <div class="flex flex-col md:flex-row items-start gap-12 max-w-7xl mx-auto">

        <div class="flex flex-col items-center w-full md:w-1/3 shrink-0">
            
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

        <div class="w-full md:w-2/3 overflow-x-auto">
            
            <Table hoverable={true} class="w-full text-left">
                <TableHead>
                    <TableHeadCell>ID</TableHeadCell>
                    <TableHeadCell>Start Date</TableHeadCell>
                    <TableHeadCell>End Date</TableHeadCell>
                    <TableHeadCell>Start Time</TableHeadCell>
                    <TableHeadCell>End Time</TableHeadCell>
                    <TableHeadCell>Duration</TableHeadCell>
                </TableHead>
                
                <TableBody>
                    {#each gameSessions as session}
                        <TableBodyRow class="bg-primary! border-blue-500! hover:bg-gray-800! cursor-pointer transition-colors">
                            <TableBodyCell>{session.sessionId}</TableBodyCell>
                            <TableBodyCell>{formatDate(session.startTs)}</TableBodyCell>
                            <TableBodyCell>{formatDate(session.endTs)}</TableBodyCell>
                            <TableBodyCell>{formatTime(session.startTs)}</TableBodyCell>
                            <TableBodyCell>{formatTime(session.endTs)}</TableBodyCell>
                            <TableBodyCell>{formatDuration(session.durationSeconds)}</TableBodyCell>
                        </TableBodyRow>
                    {/each}
                </TableBody>
            </Table>
            
        </div>

    </div>
</main>
