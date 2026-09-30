<script lang="ts">
    import { Textarea } from "$lib/components/ui/textarea";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import * as Select from "$lib/components/ui/select";
    import * as Table from "$lib/components/ui/table";
    import { ArrowUpDown, ArrowUp, ArrowDown } from "@lucide/svelte";
    import { createSvelteTable, getCoreRowModel, getSortedRowModel, getPaginationRowModel, type ColumnDef, type SortingState, type PaginationState } from "$lib/table";
    import { fade } from "svelte/transition";

    import {
        formatDate,
        formatTime,
        formatDuration,
        formatLocaleDate,
    } from "$lib/timeFormatting";

    import type { Session } from "$lib/types";



    import { invoke } from "@tauri-apps/api/core";



    // Sessions prop from gameInfo page.
    let { sessions = $bindable() }: { sessions: Session[] } = $props();

    // Container for the single session view.
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

    const columns: ColumnDef<TableSession>[] = [
        { accessorKey: "displayId", header: "ID" },
        { id: "startDate", accessorFn: (s) => new Date(s.startTs).getTime(), header: "Start Date" },
        { id: "endDate", accessorFn: (s) => new Date(s.endTs).getTime(), header: "End Date" },
        { id: "startTime", accessorFn: (s) => secondsOfDay(s.startTs), header: "Start Time" },
        { id: "endTime", accessorFn: (s) => secondsOfDay(s.endTs), header: "End Time" },
        { accessorKey: "durationSeconds", header: "Duration" },
    ];

    function secondsOfDay(timestamp: string) {
        const date = new Date(timestamp);
        return date.getHours() * 3600 + date.getMinutes() * 60 + date.getSeconds();
    }

    let sorting = $state<SortingState>([]);
    let pagination = $state<PaginationState>({ pageIndex: 0, pageSize: 10 });
    const table = createSvelteTable({
        get data() { return sortedSessions; },
        columns,
        getRowId: (row) => String(row.sessionId),
        state: {
            get sorting() { return sorting; },
            get pagination() { return pagination; },
        },
        onSortingChange: (updater) => {
            sorting = typeof updater === "function" ? updater(sorting) : updater;
            table.setPageIndex(0);
        },
        onPaginationChange: (updater) => {
            pagination = typeof updater === "function" ? updater(pagination) : updater;
        },
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
    });

    function getSingleSession(foundSession: TableSession) {
        session = foundSession;
        selected = true;
        editNotesFlag = false;
        successMsg = "";
        errorMsg = "";
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

</script>

{#if !selected}
    {#if sessions.length === 0}
        <p class="py-8 text-muted-foreground">No sessions recorded yet.</p>
    {:else}
        <div class="rounded-lg border">
            <Table.Root>
                <Table.Caption>Choose a session ID to view its details and notes.</Table.Caption>
                <Table.Header>
                    {#each table.getHeaderGroups() as group}
                        <Table.Row>
                            {#each group.headers as header}
                                <Table.Head aria-sort={header.column.getIsSorted() === "asc" ? "ascending" : header.column.getIsSorted() === "desc" ? "descending" : "none"}>
                                    <Button variant="ghost" size="sm" onclick={header.column.getToggleSortingHandler()}>
                                        {header.column.columnDef.header}
                                        {#if header.column.getIsSorted() === "asc"}<ArrowUp />
                                        {:else if header.column.getIsSorted() === "desc"}<ArrowDown />
                                        {:else}<ArrowUpDown />{/if}
                                    </Button>
                                </Table.Head>
                            {/each}
                        </Table.Row>
                    {/each}
                </Table.Header>
                <Table.Body>
                    {#each table.getRowModel().rows as row (row.id)}
                        <!-- A native button supplies keyboard access; clicking anywhere in the row is a convenience. -->
                        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
                        <Table.Row class="cursor-pointer" onclick={() => getSingleSession(row.original)}>
                            <Table.Cell><Button variant="ghost" size="sm" onclick={() => getSingleSession(row.original)} aria-label={"View session " + row.original.displayId}>{row.original.displayId}</Button></Table.Cell>
                            <Table.Cell>{formatLocaleDate(row.original.startTs)}</Table.Cell>
                            <Table.Cell>{formatLocaleDate(row.original.endTs)}</Table.Cell>
                            <Table.Cell>{formatTime(row.original.startTs)}</Table.Cell>
                            <Table.Cell>{formatTime(row.original.endTs)}</Table.Cell>
                            <Table.Cell class="font-mono tabular-nums">{formatDuration(row.original.durationSeconds)}</Table.Cell>
                        </Table.Row>
                    {/each}
                </Table.Body>
            </Table.Root>
        </div>
        <div class="mt-4 flex flex-wrap items-center justify-between gap-3">
            <div class="flex items-center gap-2">
                <Label for="page-size">Rows per page</Label>
                <Select.Root type="single" value={String(pagination.pageSize)} onValueChange={(value) => { if (value) { table.setPageSize(Number(value)); table.setPageIndex(0); } }}>
                    <Select.Trigger id="page-size" class="w-20">{pagination.pageSize}</Select.Trigger>
                    <Select.Content>{#each [5, 10, 15, 20, 25] as size}<Select.Item value={String(size)}>{size}</Select.Item>{/each}</Select.Content>
                </Select.Root>
            </div>
            <p class="text-sm text-muted-foreground">Page {pagination.pageIndex + 1} of {Math.max(1, table.getPageCount())} · {sessions.length} sessions</p>
            <div class="flex gap-2">
                <Button variant="outline" size="sm" disabled={!table.getCanPreviousPage()} onclick={() => table.previousPage()}>Previous</Button>
                <Button variant="outline" size="sm" disabled={!table.getCanNextPage()} onclick={() => table.nextPage()}>Next</Button>
            </div>
        </div>
    {/if}
<!--SINGLE SESSION VIEW-->
{:else if selected}
    <div class="mb-6">
        <Button variant="link"
            class="px-0"
            onclick={() => (
                (selected = false), 
                (successMsg = ""), 
                (errorMsg = "")
            )}
        >
            ← Back to Session List
        </Button>
    </div>

    <!--Displays the details of single session-->
    <div class="text-foreground flex flex-col gap-8" in:fade={{ duration: 100 }}>
        <div class="flex flex-row flex-wrap gap-8">
            <!--Start Date-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Date</h2>
                <hr class="w-full mt-1 mb-2 border-border" />
                <p>{formatDate(session.startTs!)}</p>
            </div>

            <!--End Date-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Date</h2>
                <hr class="w-full mt-1 mb-2 border-border" />
                <p>{formatDate(session.endTs!)}</p>
            </div>

            <!--Start Time-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Start Time</h2>
                <hr class="w-full mt-1 mb-2 border-border" />
                <p>{formatTime(session.startTs!)}</p>
            </div>

            <!--End Time-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">End Time</h2>
                <hr class="w-full mt-1 mb-2 border-border" />
                <p>{formatTime(session.endTs!)}</p>
            </div>

            <!--Duration in HH::MM::SS-->
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">Duration</h2>
                <hr class="w-full mt-1 mb-2 border-border" />
                <p>{formatDuration(session.durationSeconds!)}</p>
            </div>
        </div>

        <div>
            <!--Session Notes. Displayed if user isn't editing notes-->
            {#if !editNotesFlag}
                <h2 class="text-2xl font-bold">Session Notes:</h2>
                <p class="mt-2 text-lg text-muted-foreground whitespace-pre-wrap">
                    {session.notes ?? "No notes recorded"}
                </p>
                <Button variant="link"
                    class="px-0"
                    onclick={() => (
                        (editNotesFlag = true),
                        (updatedNotes = session.notes ?? ""),
                        (errorMsg = ""),
                        (successMsg = "")
                    )}
                >
                    Edit Session Notes
                </Button>

            <!--Edit notes section. Displayed if user is editing notes-->
            {:else if editNotesFlag}
                <Textarea
                    id="notes-input"
                    class="mt-3 w-full min-h-64"
                    aria-label="Session notes"
                    placeholder="Enter session notes..."
                    bind:value={updatedNotes}
                />

                <Button variant="link"
                    class="px-0"
                    onclick={() => (editNotesFlag = false)}
                >
                    Cancel Editing
                </Button>

                <Button variant="link"
                    class="px-0"
                    onclick={() =>
                        editNotes(Number(session.sessionId), updatedNotes)}
                >
                    Finish Editing
                </Button>
            {/if}

            {#if successMsg.length > 0}
                <p class="text-base font-semibold">{successMsg}</p>
            {:else if errorMsg.length > 0}
                <p class="text-base font-semibold">{errorMsg}</p>
            {/if}
        </div>
    </div>
{/if}
