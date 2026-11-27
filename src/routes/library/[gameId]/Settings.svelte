
<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import Input from "#lib/components/ui/input/input.svelte";
    import Button from "#lib/components/ui/button/button.svelte";
    import Label from "#lib/components/ui/label/label.svelte";

    let { gameId }: { gameId: number } = $props();

    let loading = $state(false);
    let saving = $state(false);
    let error = $state("");

    let processName = $state("");
    let processNameEdited = $state("");
    let editing = $state(false);

    async function getProcessNames() {
        loading = true;
        error = "";

        try {
            const names = await invoke<string[]>(
                "get_processes",
                { gameId }
            );

            processName = names[0] ?? "";
            processNameEdited = processName;

        } catch (err) {
            error = String(err);
            console.error(err);
        } finally {
            loading = false;
        }
    }

    async function insertProcess() {
        saving = true;
        error = "";

        try {
            await invoke("insert_process_name", {
                gameId,
                processName: processNameEdited.trim()
            });

            await getProcessNames();
            editing = false;

        } catch (err) {
            error = String(err);
            console.error(err);
        } finally {
            saving = false;
        }
    }

    function startEditing() {
        processNameEdited = processName;
        editing = true;
    }

    function cancelEditing() {
        processNameEdited = processName;
        editing = false;
        error = "";
    }

    onMount(() => {
        getProcessNames();
    });
</script>

{#if loading}
    <div class="justify-items-center">
        <p>Loading settings...</p>
    </div>
{:else}
    <Label class="text-lg border-b-2">
        Process Name
    </Label>

    <div class="flex items-center max-w-lg p-2 gap-2">
        {#if editing || !processName}
            <Input
                type="text"
                bind:value={processNameEdited}
                placeholder="Process name..."
            />

            <Button
                class="cursor-pointer"
                variant="outline"
                disabled={saving || !processNameEdited.trim()}
                onclick={insertProcess}
            >
                {saving ? "Saving..." : "Submit"}
            </Button>

            {#if editing}
                <Button
                    class="cursor-pointer"
                    variant="outline"
                    onclick={cancelEditing}
                >
                    Cancel
                </Button>
            {/if}
        {:else}
            <p class="text-balance">{processName}</p>

            <Button
                class="cursor-pointer"
                variant="outline"
                onclick={startEditing}
            >
                Edit
            </Button>
        {/if}
    </div>

    {#if error}
        <p class="text-red-500 p-2">{error}</p>
    {/if}
{/if}
