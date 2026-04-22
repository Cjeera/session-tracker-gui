<script lang="ts">
    import { Textarea } from "flowbite-svelte";

    import {
        Table,
        TableBody,
        TableBodyCell,
        TableBodyRow,
        TableHead,
        TableHeadCell,
    } from "@flowbite-svelte-plugins/datatable";

    import {
        formatDate,
        formatTime,
        formatDuration,
        formatLocaleDate,
    } from "$lib/timeFormatting";

    import type { Session } from "$lib/types";

    import type { DataTable } from "@flowbite-svelte-plugins/datatable";

    import { invoke } from "@tauri-apps/api/core";

    import { ListPlaceholder } from "flowbite-svelte";

    // Sessions prop from gameInfo page.
    let { sessions = $bindable() }: { sessions: Session[] } = $props();

    // State for the single session view.
    let session = $state<Partial<Session>>({});

    // State for single session view display.
    let selected = $state(false);

    // State for edit notes display.
    let editNotesFlag = $state(false);

    // State for the updated session notes.
    let updatedNotes = $state("");

    // State for success messages for editNotes.
    let successMsg = $state("");

    // State for error messages for editNotes.
    let errorMsg = $state("");

    type TableSession = Session & { displayId: number };

    let sortedSessions = $derived.by(() => {
        // add the sequential ID to a copy of the sessions
        let sessionsWithId: TableSession[] = sessions.map((s, i) => ({
            ...s,
            displayId: i + 1,
        }));
        return sessionsWithId;
    });

    /** Opens the detailed view for a specific session.*/
    function getSingleSession(index: number, table: DataTable) {
        // The row data is obtained.
        const row = table.data.data[index];

        // The session ID of the row is taken from the first column.
        const sessionId = Number(row.cells[0].text);

        // Attempts to find the specific session by trying to match the ID taken from the column against the IDs in sortedSessions.
        const foundSession = sortedSessions.find(
            (session) => session.displayId === sessionId,
        );

        // If no session found, returns.
        if (!foundSession) return;

        // foundSession is assigned to found session.
        session = foundSession;

        // Selected is set to true, which will display the session view.
        selected = true;
    }

    /** Function for editing session notes*/
    async function editNotes(sessionId: number, updatedNotes: string) {
        errorMsg = "";

        try {
            // Backend function is called with sessionId and updatedNotes sent as arguments.
            await invoke("edit_notes", { sessionId, updatedNotes });

            // Session notes from the single session is updated with the new notes.
            session.notes = updatedNotes;

            // The original sessions list is updated with the new notes.
            const index = sessions.findIndex((s) => s.sessionId === sessionId);
            if (index !== -1) {
                sessions[index].notes = updatedNotes;
            }

            // Success message is displayed to the user.
            successMsg = "Updated Notes Successfully!";

            // Error message is emptied.
            errorMsg = "";

            // Edit notes flag set to false, will cause edit notes text area to dissapear.
            editNotesFlag = false;
        } catch (error) {
            errorMsg = "database error!";

            successMsg = "";
        }
    }
    let tableReady = $state(false);
</script>

<!--Displays the session list view if selected if false, meaning user hasn't selected an individual session-->
{#if !selected}
    {#if sessions.length === 0}
        <ListPlaceholder />
    {:else}
        <!--SESSION LIST TABLE-->
        <div
            class="transition-opacity duration-75 {tableReady
                ? 'opacity-100'
                : 'opacity-0'}"
        >
            <Table
                onInitStart={() => (tableReady = false)}
                onInitComplete={(_table) => {
                    tableReady = true;
                }}
                selectable={true}
                onSelectRow={(index, _event, table) => {
                    getSingleSession(index, table);
                }}
                class="w-full text-left bg-transparent"
                divClass="bg-transparent border-0 shadow-none relative overflow-x-auto"
                dataTableOptions={{ searchable: false, sortable: true }}
            >
                <!--SESSION TABLE COLUMNS-->
                <TableHead class="bg-transparent">
                    <TableHeadCell
                        class="bg-[#364153] text-gray-400 uppercase text-xs font-bold tracking-wider select-none transition-colors px-4 py-3"
                    >
                        ID
                    </TableHeadCell>

                    <TableHeadCell
                        class="bg-[#364153] text-gray-400 uppercase text-xs font-bold tracking-wider select-none transition-colors px-4 py-3"
                    >
                        Start Date
                    </TableHeadCell>

                    <TableHeadCell
                        class="bg-[#364153] text-gray-400 uppercase text-xs font-bold tracking-wider select-none transition-colors px-4 py-3"
                    >
                        End Date
                    </TableHeadCell>

                    <TableHeadCell
                        class="bg-[#364153] text-gray-400 uppercase text-xs font-bold tracking-wider select-none transition-colors px-4 py-3"
                    >
                        Start Time
                    </TableHeadCell>

                    <TableHeadCell
                        class="bg-[#364153] text-gray-400 uppercase text-xs font-bold tracking-wider select-none transition-colors px-4 py-3"
                    >
                        End Time
                    </TableHeadCell>

                    <TableHeadCell
                        class="bg-[#364153] text-gray-400 uppercase text-xs font-bold tracking-wider select-none transition-colors px-4 py-3"
                    >
                        Duration
                    </TableHeadCell>
                </TableHead>
                <!--SESSION TABLE ROWS-->
                <TableBody>
                    {#each sortedSessions as session}
                        <TableBodyRow
                            class="bg-primary! border-b! border-blue-500! hover:bg-gray-800! cursor-pointer transition-colors"
                        >
                            <TableBodyCell class="text-gray-400 font-semibold"
                                >{session.displayId}</TableBodyCell
                            >
                            <TableBodyCell class="text-gray-400 font-semibold"
                                >{formatLocaleDate(
                                    session.startTs,
                                )}</TableBodyCell
                            >
                            <TableBodyCell class="text-gray-400 font-semibold"
                                >{formatLocaleDate(
                                    session.endTs,
                                )}</TableBodyCell
                            >
                            <TableBodyCell class="text-gray-400 font-semibold"
                                >{formatTime(session.startTs)}</TableBodyCell
                            >
                            <TableBodyCell class="text-gray-400 font-semibold"
                                >{formatTime(session.endTs)}</TableBodyCell
                            >
                            <TableBodyCell class="text-gray-400 font-semibold"
                                >{formatDuration(
                                    session.durationSeconds,
                                )}</TableBodyCell
                            >
                        </TableBodyRow>
                    {/each}
                </TableBody>
            </Table>
        </div>
    {/if}
    <!--SINGLE SESSION VIEW-->
{:else if selected}
    <div class="mb-6">
        <button
            class="text-blue-500 hover:text-blue-400 underline cursor-pointer"
            onclick={() => (
                (selected = false), (successMsg = ""), (errorMsg = "")
            )}
        >
            ← Back to Session List
        </button>
    </div>

    <!--Displays the details of single session-->
    <div class="text-white flex flex-col gap-8">
        <div class="flex flex-row flex-wrap gap-8">
            <!--Start Date-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Date</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatDate(session.startTs!)}</p>
            </div>

            <!--End Date-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Date</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatDate(session.endTs!)}</p>
            </div>

            <!--Start Time-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Time</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatTime(session.startTs!)}</p>
            </div>

            <!--End Time-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Time</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatTime(session.endTs!)}</p>
            </div>

            <!--Duration in HH::MM::SS-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Duration</h2>
                <hr class="w-full mt-1 mb-2 border-gray-600" />
                <p>{formatDuration(session.durationSeconds!)}</p>
            </div>
        </div>

        <div>
            <!--Session Notes. Displayed if user isn't editing notes-->
            {#if !editNotesFlag}
                <h2 class="text-2xl font-bold">Session Notes:</h2>
                <p class="mt-2 text-lg text-gray-300 whitespace-pre-wrap">
                    {session.notes ?? "No notes recorded"}
                </p>
                <button
                    class="text-blue-500 hover:text-blue-400 underline cursor-pointer"
                    onclick={() => (
                        (editNotesFlag = true),
                        (updatedNotes = session.notes ?? ""),
                        (errorMsg = ""),
                        (successMsg = "")
                    )}
                >
                    Edit Session Notes
                </button>

                <!--Edit Notes. Displayed if user is editing notes-->
            {:else if editNotesFlag}
                <Textarea
                    id="notes-input"
                    class="p-2 rounded mt-3 w-lg h-64 placeholder-blue-400!"
                    classes={{ wrapper: "!bg-primary" }}
                    placeholder="Enter session notes..."
                    bind:value={updatedNotes}
                />

                <button
                    class="text-blue-500 hover:text-blue-400 underline cursor-pointer"
                    onclick={() => (editNotesFlag = false)}
                >
                    Cancel Editing
                </button>

                <button
                    class="text-blue-500 hover:text-blue-400 underline cursor-pointer"
                    onclick={() =>
                        editNotes(Number(session.sessionId), updatedNotes)}
                >
                    Finish Editing
                </button>
            {/if}

            {#if successMsg.length > 0}
                <p class="text-base font-semibold">{successMsg}</p>
            {:else if errorMsg.length > 0}
                <p class="text-base font-semibold">{errorMsg}</p>
            {/if}
        </div>
    </div>
{/if}
