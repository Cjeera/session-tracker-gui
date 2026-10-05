<script>
    import { onMount } from 'svelte';
    import { page } from "$app/state";
    import "../app.css";
    import { Button } from "#lib/components/ui/button/index.ts";
    
    // Import the Tauri plugins
    import { check } from '@tauri-apps/plugin-updater';
    import { ask, message } from '@tauri-apps/plugin-dialog';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { tracker } from "./sessionTracker.svelte.ts";
    import { getCurrentWindow } from "@tauri-apps/api/window";

    let { children } = $props();
    let activeUrl = $derived(page.url.pathname);

    $effect(() => {
        let unlisten = () => {};

        async function setupCloseListener() {
            unlisten = await getCurrentWindow().onCloseRequested(async (event) => {
                // Check if the tracker is running
                if (tracker.stopwatchDisplay.length > 0) {
                
                    // Prevent the window from closing immediately
                    event.preventDefault();

                    // Ask the user
                    const confirmed = await ask("Tracker is currently running. Close application?", {
                        title: "Session Tracker GUI",
                        kind: "warning"
                    });

                    // If they confirm, bypass the interceptor and destroy the window
                    if (confirmed) {
                        await getCurrentWindow().destroy(); 
                    }
                }
            });
        }
        setupCloseListener();
        // Cleanup the event listener if the Svelte component is destroyed
        return () => unlisten();
    });

 
    // Run the update check once when the app starts
    onMount(async () => {
        try {
            const update = await check();
            
            if (update) {
                // Dialogue box is displayed for the user to update the app.                
                const wantsToUpdate = await ask(
                    `Version ${update.version} is available!\n\nRelease notes:\n${update.body}\n\nWould you like to install it now?`, 
                    {
                        title: 'Update Available!',
                        kind: 'info',
                        okLabel: 'Update Now',
                        cancelLabel: 'Later'
                    }
                );

                if (wantsToUpdate) {
                    // Downloads and installs the update silently in the background
                    await update.downloadAndInstall();
                    
                    // Notify the user and restart
                    await message('Update installed! The app will now restart.', { title: 'Success' });
                    await relaunch();
                }
            }
        } catch (error) {
            console.error('Failed to check for updates:', error);
        }
    });
</script>

<header class="border-b bg-card">
    <nav aria-label="Main navigation" class="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-3 p-4">
        <a href="/" class="text-xl font-semibold">Session Tracker GUI</a>
        {#if tracker.stopwatchDisplay.length > 0 && activeUrl.includes("/library")}
            <span class="font-mono text-xl tabular-nums" role="timer">{tracker.stopwatchDisplay}</span>
        {/if}
        <div class="flex gap-2">
            <Button href="/" variant={activeUrl === "/" ? "secondary" : "ghost"} aria-current={activeUrl === "/" ? "page" : undefined}>Session Tracker</Button>
            <Button href="/library" variant={activeUrl.startsWith("/library") ? "secondary" : "ghost"} aria-current={activeUrl.startsWith("/library") ? "page" : undefined}>Library</Button>
        </div>
    </nav>
</header>

{@render children()}
