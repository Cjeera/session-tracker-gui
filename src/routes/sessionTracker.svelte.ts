import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { Process } from "#lib/types.svelte.js";
import { toast } from "svelte-sonner";
import {
        isPermissionGranted,
        requestPermission,
        sendNotification,
        } from '@tauri-apps/plugin-notification';

// SessionRust is serialized by Rust using snake_case field names.
type SessionPayload = {
    game: string;
    start_ts: string;
    end_ts: string;
    duration_seconds: number;
    notes: string | null;
    process_name: string;
};

// A class containing all session tracking logic.
class SessionTracker {
    unlistenStopwatch: UnlistenFn | null = null;
    unlistenSessionEnd: UnlistenFn | null = null;
    unlistenSessionStateChange: UnlistenFn | null = null;

    seconds = $state(0);
    minutes = $state(0);
    hours = $state(0);
    stopwatchDisplay = $state("");
    searchResults = $state<Process[]>([]);
    gameInput = $state("");
    newGameInput = $state("");
    headerMessage = $state("Enter a game title to get started");
    sessionNotes = $state("");
    successMsg = $state("");
    errorMsg = $state("");
    errorFlag = $state(false);
    gameFound = $state(false);
    paused = $state(false);
    searchSuccessful = $state(false);
    starting = $state(false);
    saving = $state(false);
    stopping = $state(false);
    sessionData = $state<Partial<SessionPayload>>({});
    pid = $state(0);
    StopwatchPayload = $state({ elapsedMs: 0 });

    private clearListeners = () => {
        this.unlistenStopwatch?.();
        this.unlistenSessionEnd?.();
        this.unlistenSessionStateChange?.();
        this.unlistenStopwatch = null;
        this.unlistenSessionEnd = null;
        this.unlistenSessionStateChange = null;
    };

    searchProcesses = async (event: Event) => {
        event.preventDefault();
        if (this.gameFound || this.starting || this.saving) return;
        this.errorFlag = false;
        this.searchResults = [];
        this.stopwatchDisplay = "";
        this.successMsg = "";
        this.searchSuccessful = false;
        if (this.gameInput.length <= 1) {
            this.errorFlag = true;
            this.errorMsg = "Enter a name at least 2 characters long";
            return;
        }
        try {
            const results = await invoke<Process[]>("search_processes", { gameInput: this.gameInput });
            if (this.gameFound || this.starting) return;
            this.searchResults = results;
            this.searchSuccessful = true;
        } catch (error) {
            this.errorFlag = true;
            this.errorMsg = String(error);
            console.error(error);
        }
    };

    trackSession = async (process: Process) => {
        // Both manual clicks and automatic detections use this function.
        if (this.starting || this.gameFound || this.saving) return;
        this.starting = true;
        this.paused = false;
        this.stopping = false;
        this.searchSuccessful = false;
        this.errorFlag = false;
        this.errorMsg = "";
        this.successMsg = "";
        this.pid = process.pid;
        this.gameInput = process.name;
        this.clearListeners();

        try {
            this.unlistenSessionStateChange = await listen<string>("session-state-change", (event) => {
                if (event.payload === "paused") this.paused = true;
                else if (event.payload === "resumed") this.paused = false;
            });
            this.unlistenStopwatch = await listen<{ elapsedMs: number }>("stopwatch-tick", (event) => {
                this.StopwatchPayload.elapsedMs = event.payload.elapsedMs;
                const totalSeconds = Math.floor(event.payload.elapsedMs / 1000);
                this.hours = Math.floor(totalSeconds / 3600);
                this.minutes = Math.floor(totalSeconds / 60) % 60;
                this.seconds = totalSeconds % 60;
                this.stopwatchDisplay = "Elapsed Time:\n" +
                    String(this.hours).padStart(2, "0") + ":" +
                    String(this.minutes).padStart(2, "0") + ":" +
                    String(this.seconds).padStart(2, "0");
            });
            this.unlistenSessionEnd = await listen<SessionPayload>("session-ended", async (event) => {
                let permissionGranted = await isPermissionGranted();
                if (!permissionGranted) {
                    const permission = await requestPermission();
                    permissionGranted = permission === 'granted';
                }

                if (permissionGranted) {
                    sendNotification({ title: 'Session Ended', body: `${this.stopwatchDisplay}` });
                }

                this.sessionData = event.payload;
                this.headerMessage = "Session ended!";
                this.paused = false;
                this.stopping = false;
            });

            // Set up the listeners before starting the backend worker.
            this.stopwatchDisplay = "Elapsed Time: 00:00:00";
            this.headerMessage = "Currently Tracking " + this.gameInput + "\n(PID " + this.pid + ")";
            this.gameFound = true;
            await invoke("start_tracker", { gameInput: this.gameInput, pid: this.pid });
        } catch (error) {
            this.clearListeners();
            this.errorFlag = true;
            this.errorMsg = String(error);
            this.gameFound = false;
            this.stopwatchDisplay = "";
            this.headerMessage = "Enter a game title to get started";
        } finally {
            this.starting = false;
        }
    };

    pauseSession = async () => {
        if (this.stopping || this.starting) return;
        try {
            await invoke("toggle_pause");
        } catch (error) {
            this.errorMsg = String(error);
            toast.error(String(error));
        }
    };

    resumeSession = async () => {
        if (this.stopping || this.starting) return;
        try {
            await invoke("toggle_resume");
        } catch (error) {
            this.errorMsg = String(error);
            toast.error(String(error));
        }
    };

    userStopSession = async () => {
        if (this.stopping || this.starting) return;
        this.stopping = true;
        try {
            await invoke("toggle_end");
        } catch (error) {
            this.stopping = false;
            this.errorMsg = String(error);
            toast.error(String(error));
        }
    };

    endSession = async (event: Event) => {
        event.preventDefault();
        if (this.saving || !Object.keys(this.sessionData).length) return;
        this.saving = true;
        this.errorFlag = false;
        this.errorMsg = "";
        try {
            // Do not mutate the original session on a failed save.
            const data = { ...this.sessionData };
            if (this.newGameInput.trim().length > 0) data.game = this.newGameInput.trim();
            await invoke("end_tracker", { sessionNotes: this.sessionNotes, sessionData: data });
            this.clearListeners();
            this.successMsg = "Session data saved to database!";
            this.sessionData = {};
            this.gameInput = "";
            this.newGameInput = "";
            this.sessionNotes = "";
            this.headerMessage = "Enter a game title to get started";
            this.gameFound = false;
            this.searchResults = [];
            this.paused = false;
            this.stopping = false;
            this.stopwatchDisplay = "";
        } catch (error) {
            this.errorFlag = true;
            this.errorMsg = String(error);
            console.error("Failed to save session:", error);
            toast.error(String(error));
        } finally {
            this.saving = false;
        }
    };
}

export const tracker = new SessionTracker();
