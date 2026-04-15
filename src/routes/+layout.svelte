<script>
    import { onMount } from 'svelte';
    import { page } from "$app/state";
    import { Navbar, NavBrand, NavLi, NavUl, NavHamburger } from "flowbite-svelte";
    
    // Import the Tauri plugins
    import { check } from '@tauri-apps/plugin-updater';
    import { ask, message } from '@tauri-apps/plugin-dialog';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { tracker } from "./sessionTracker.svelte.js";
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

<!--NAVIGATION BAR-->
<Navbar fluid={true}>
    <NavBrand href="/">
        <span class="self-center whitespace-nowrap text-xl font-semibold text-white">Session Tracker GUI</span>
    </NavBrand>
    <NavHamburger/>
    <NavUl {activeUrl}>
        <NavLi href="/">Session Tracker</NavLi>
        <NavLi href="/library">Library</NavLi>
    </NavUl>
</Navbar>

{@render children()}