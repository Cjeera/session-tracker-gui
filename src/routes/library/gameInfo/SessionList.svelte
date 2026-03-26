<script lang="ts">
    import {
        Table,
        TableBody,
        TableBodyCell,
        TableBodyRow,
        TableHead,
        TableHeadCell,
    } from "flowbite-svelte";
    import {
        formatDate,
        formatTime,
        formatDuration,
        formatLocaleDate,
    } from "./timeFormatting";

    import type { Session } from "./types";

    // Sessions prop from gameInfo page.
    let { sessions }: { sessions: Session[] } = $props();

    // State for the single session view.
    let session = $state<Partial<Session>>({});

    let selected = $state(false);

    type TableSession = Session & { displayId: number };

    type SortKey = keyof TableSession | "";

    // State for the table sorting.
    let sortKey = $state<SortKey>("");
    let sortDirection = $state<"asc" | "desc">("asc");

    /** Handles the sorting logic when a user clicks a table header. Updates the sort key and toggles the ascending/descending direction.*/
    function handleSort(key: SortKey) {
        // Checks if the user clicked the column that is already being sorted.
        if (sortKey === key) {
            // Toggles direction to descending if currently ascending.
            if (sortDirection === "asc") {
                sortDirection = "desc";
            }
            // Disables sorting entirely if already descending.
            else {
                sortKey = "";
                sortDirection = "asc";
            }
        }
        // If a new column is clicked, it sets the key and defaults to ascending.
        else {
            sortKey = key;
            sortDirection = "asc";
        }
    }

    let sortedSessions = $derived.by(() => {
        // add the sequential ID to a copy of the sessions
        let sessionsWithId: TableSession[] = sessions.map((s, i) => ({
            ...s,
            displayId: i + 1,
        }));

        // Return early if no sort key
        if (!sortKey) return sessionsWithId;

        const currentKey = sortKey as keyof TableSession;

        // Sort the new array (using displayId if that's the sortKey)
        return [...sessionsWithId].sort((a, b) => {
            let valA = a[currentKey] as any;
            let valB = b[currentKey] as any;

            if (valA < valB) return sortDirection === "asc" ? -1 : 1;
            if (valA > valB) return sortDirection === "asc" ? 1 : -1;
            return 0;
        });
    });
    /** Opens the detailed view for a specific session.*/
    function getSingleSession(selectedSession: TableSession) {
        // The clicked session data is assigned to the state object.
        session = selectedSession;

        // The view is toggled from the table to the single session detail.
        selected = true;
    }
</script>

<!--Displays the session list view if selected if false, meaning user hasn't selected an individual session-->
{#if sessions.length === 0}
    <h5 class="pb-1 text-2xl leading-none font-bold text-white">
        No Sessions Recorded Yet!
    </h5>
{:else if !selected}
    <Table
        hoverable={true}
        classes={{
            div: "bg-primary! border-0 shadow-none relative overflow-x-auto",
        }}
        class="w-full text-left"
    >
        <!-- Displays the table headers, displaying an arrow next to them depending on sort state-->
        <TableHead>
            <TableHeadCell
                onclick={() => handleSort("displayId")}
                class="cursor-pointer hover:text-blue-400 select-none transition-colors"
            >
                ID {sortKey === "displayId"
                    ? sortDirection === "asc"
                        ? "↑"
                        : "↓"
                    : ""}
            </TableHeadCell>

            <TableHeadCell
                onclick={() => handleSort("startTs")}
                class="cursor-pointer hover:text-blue-400 select-none transition-colors"
            >
                Start Date {sortKey === "startTs"
                    ? sortDirection === "asc"
                        ? "↑"
                        : "↓"
                    : ""}
            </TableHeadCell>

            <TableHeadCell
                onclick={() => handleSort("endTs")}
                class="cursor-pointer hover:text-blue-400 select-none transition-colors"
            >
                End Date {sortKey === "endTs"
                    ? sortDirection === "asc"
                        ? "↑"
                        : "↓"
                    : ""}
            </TableHeadCell>

            <TableHeadCell
                onclick={() => handleSort("startTs")}
                class="cursor-pointer hover:text-blue-400 select-none transition-colors"
            >
                Start Time {sortKey === "startTs"
                    ? sortDirection === "asc"
                        ? "↑"
                        : "↓"
                    : ""}
            </TableHeadCell>

            <TableHeadCell
                onclick={() => handleSort("endTs")}
                class="cursor-pointer hover:text-blue-400 select-none transition-colors"
            >
                End Time {sortKey === "endTs"
                    ? sortDirection === "asc"
                        ? "↑"
                        : "↓"
                    : ""}
            </TableHeadCell>

            <TableHeadCell
                onclick={() => handleSort("durationSeconds")}
                class="cursor-pointer hover:text-blue-400 select-none transition-colors"
            >
                Duration {sortKey === "durationSeconds"
                    ? sortDirection === "asc"
                        ? "↑"
                        : "↓"
                    : ""}
            </TableHeadCell>
        </TableHead>

        <TableBody>
            <!--Displays every entry in sortedSessions as a row-->
            {#each sortedSessions as session}
                <!--If user clicks on a session row, getSingleSession is called-->
                <TableBodyRow
                    class="bg-primary! border-b border-blue-500! hover:bg-gray-800! cursor-pointer transition-colors"
                    onclick={() => getSingleSession(session)}
                >
                    <TableBodyCell>{session.displayId}</TableBodyCell>

                    <TableBodyCell
                        >{formatLocaleDate(session.startTs)}</TableBodyCell
                    >
                    <TableBodyCell
                        >{formatLocaleDate(session.endTs)}</TableBodyCell
                    >
                    <TableBodyCell>{formatTime(session.startTs)}</TableBodyCell>
                    <TableBodyCell>{formatTime(session.endTs)}</TableBodyCell>
                    <TableBodyCell
                        >{formatDuration(
                            session.durationSeconds,
                        )}</TableBodyCell
                    >
                </TableBodyRow>
            {/each}
        </TableBody>
    </Table>
    <!--Displays the details of single session-->
{:else}
    <div class="mb-6">
        <button
            class="text-blue-500 hover:text-blue-400 underline cursor-pointer"
            onclick={() => (selected = false)}
        >
            ← Back to Session List
        </button>
    </div>

    <div class="text-white flex flex-col gap-8">
        <div class="flex flex-row flex-wrap gap-8">
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Date</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatDate(session.startTs!)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Date</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatDate(session.endTs!)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Time</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatTime(session.startTs!)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Time</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatTime(session.endTs!)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Duration</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatDuration(session.durationSeconds!)}</p>
            </div>
        </div>

        <div>
            <h2 class="text-2xl font-bold">Session Notes:</h2>
            <p class="mt-2 text-lg text-gray-300">
                {session.notes || "No session notes available."}
            </p>
        </div>
    </div>
{/if}
