<script>
    import {
        Table,
        TableBody,
        TableBodyCell,
        TableBodyRow,
        TableHead,
        TableHeadCell,
    } from "flowbite-svelte"; 
    import { formatDate, formatTime, formatDuration } from "./timeFormatting.js";

    // Component props
    let { sessions } = $props();

    // State for the single session view
    let session = $state({});
    
    let selected = $state(false);

    // State for the table sorting
    let sortKey = $state(null);
    let sortDirection = $state("asc");

    /** * Handles the sorting logic when a user clicks a table header. 
     * Updates the sort key and toggles the ascending/descending direction.
     */
    function handleSort(key) {
        // Checks if the user clicked the column that is already being sorted.
        if (sortKey === key) {
            // Toggles direction to descending if currently ascending.
            if (sortDirection === "asc") {
                sortDirection = "desc";
            } 
            // Disables sorting entirely if already descending.
            else {
                sortKey = null; 
                sortDirection = "asc";
            }
        } 
        // If a new column is clicked, it sets the key and defaults to ascending.
        else {
            sortKey = key;
            sortDirection = "asc";
        }
    }

    // A reactively derived array that automatically resorts whenever the sortKey or sortDirection changes.
    let sortedSessions = $derived.by(() => {
        // Returns the raw array early if no sorting key is active.
        if (!sortKey) return sessions;

        // A shallow copy of the sessions array is created and sorted to avoid mutating the original prop.
        return [...sessions].sort((a, b) => {
            let valA = a[sortKey];
            let valB = b[sortKey];

            // The values are subtracted to determine their chronological or numerical order based on direction.
            return sortDirection === "asc" ? valA - valB : valB - valA;
        });
    });

    /** * Opens the detailed view for a specific session.
     */
    function getSingleSession(selectedSession) {
        // The clicked session data is assigned to the state object.
        session = selectedSession;
        
        // The view is toggled from the table to the single session detail.
        selected = true;
    }
</script>
{#if !selected}
    <Table 
        hoverable={true} 
        divClass="bg-primary! border-0 shadow-none relative overflow-x-auto" 
        class="w-full text-left text-white"
    >
        <TableHead>
            <TableHeadCell onclick={() => handleSort('sessionId')} class="cursor-pointer hover:text-blue-400 select-none transition-colors">
                ID {sortKey === 'sessionId' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
            </TableHeadCell>

            <TableHeadCell onclick={() => handleSort('startTs')} class="cursor-pointer hover:text-blue-400 select-none transition-colors">
                Start Date {sortKey === 'startTs' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
            </TableHeadCell>

            <TableHeadCell onclick={() => handleSort('endTs')} class="cursor-pointer hover:text-blue-400 select-none transition-colors">
                End Date {sortKey === 'endTs' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
            </TableHeadCell>

            <TableHeadCell onclick={() => handleSort('startTs')} class="cursor-pointer hover:text-blue-400 select-none transition-colors">
                Start Time {sortKey === 'startTs' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
            </TableHeadCell>

            <TableHeadCell onclick={() => handleSort('endTs')} class="cursor-pointer hover:text-blue-400 select-none transition-colors">
                End Time {sortKey === 'endTs' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
            </TableHeadCell>

            <TableHeadCell onclick={() => handleSort('durationSeconds')} class="cursor-pointer hover:text-blue-400 select-none transition-colors">
                Duration {sortKey === 'durationSeconds' ? (sortDirection === 'asc' ? '↑' : '↓') : ''}
            </TableHeadCell>
        </TableHead>

        <TableBody>
            {#each sortedSessions as session}
                <TableBodyRow
                    class="bg-primary! border-b border-blue-500! hover:bg-gray-800! cursor-pointer transition-colors"
                    onclick={() => getSingleSession(session)}
                >
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

{:else}
    <div class="mb-6">
        <button 
            class="text-blue-500 hover:text-blue-400 underline cursor-pointer" 
            onclick={() => selected = false}
        >
            ← Back to Session List
        </button>
    </div>

    <div class="text-white flex flex-col gap-8">
        <div class="flex flex-row flex-wrap gap-8">
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Date</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600">
                <p>{formatDate(session.startTs)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Date</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600">
                <p>{formatDate(session.endTs)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Time</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600">
                <p>{formatTime(session.startTs)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Time</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600">
                <p>{formatTime(session.endTs)}</p>
            </div>

            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Duration</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600">
                <p>{formatDuration(session.durationSeconds)}</p>
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