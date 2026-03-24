<script>
    import {
        Card,
        Button,
        Tabs,
        TabItem,
        Timeline,
        TimelineItem,
        Activity,
        ActivityItem,
        Group,
        GroupItem,
    } from "flowbite-svelte";
    import { Table } from "@flowbite-svelte-plugins/datatable";
    import { Chart } from "@flowbite-svelte-plugins/chart";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    let errorMsg = $state("");
    let games = $state([]);
    let success = $state(false);

    async function getGames() {
        errorMsg = "";
        success = false;
        games = [];

        try {
            games = await invoke("get_game_list");
            success = true;
        } catch (error) {
            errorMsg = error;
            console.error("Failed to load games:", error);
        }
    }

    onMount(() => {
        getGames();
    });
</script>

<main class="min-h-screen text-white p-4">
    <div
        class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-6 justify-center items-center"
    >
        {#if !success}
            <h1>Games not found!</h1>
        {:else}
            {#each games as game}
                <Card
                    href={`/library/gameInfo?id=${game.gameId}`}
                    class="cursor-pointer relative overflow-hidden group border-none shadow-lg"
                >
                    <img
                        src={game.coverPath || "placeholder.avif"}
                        alt={game.title}
                        class="w-full h-full object-cover aspect-3/4"
                    />

                    <div
                        class="absolute inset-0 bg-black/80 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300 p-4"
                    >
                        <h5
                            class="text-xl font-bold text-center text-white tracking-tight"
                        >
                            {game.title}
                        </h5>
                    </div>
                </Card>
            {/each}
        {/if}
    </div>
</main>
