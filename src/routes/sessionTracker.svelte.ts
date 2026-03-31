import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { SessionRust, Process } from "$lib/types";

class SessionTracker {
    // Explicitly type the unlisten function
    unlistenStopwatch: UnlistenFn | null = null;

    // Seconds, minutes and hours, which will be formatted into HH:MM:SS.
    seconds = $state(0);
    minutes = $state(0);
    hours = $state(0);

    stopwatchDisplay = $state("Elapsed Time: 00:00:00");

    searchResults = $state<Process[]>([]);

    gameInput = $state("");
    errorFlag = $state(false);
    headerMessage = $state("Enter a game title to get started");
    newGameInput = $state("");
    sessionNotes = $state("");
    successMsg = $state("");
    gameFound = $state(false);
    paused = $state(false);

    sessionData = $state<Partial<SessionRust>>({});
    errorMsg = $state("");
    pid = $state(0);

    StopwatchPayload = $state({
        elapsedMs: 0,
    });

    searchSuccessful = $state(false);

    searchProcesses = async (event: Event) => {
        event.preventDefault();

        this.errorFlag = false;
        
        this.searchResults = [];

        this.stopwatchDisplay = "Elapsed Time: 00:00:00";
        this.successMsg = "";
        this.searchSuccessful = false;

        if (this.gameInput.length <= 2) {
            this.errorFlag = true;
            this.errorMsg = "Enter a name at least 3 characters long";
            return;
        }

        try {
            this.searchResults = await invoke<Process[]>("search_processes", { gameInput: this.gameInput });
            this.searchSuccessful = true;
        } catch (error) {
            this.errorFlag = true;
            // Cast unknown catch errors to string
            this.errorMsg = String(error);
            console.error(error);
        }
    }

    trackSession = async (process: Process) => {
        this.paused = false;
        this.pid = process.pid
            
        this.gameInput = process.name; 

        this.searchSuccessful = false;
        this.errorFlag = false;
        this.gameFound = true;

        this.headerMessage = "Currently Tracking " + this.gameInput + "\n(PID " + this.pid +")";

        this.unlistenStopwatch = await listen<{ elapsedMs: number }>("stopwatch-tick", (event) => {
            this.StopwatchPayload.elapsedMs = event.payload.elapsedMs;
            let totalSeconds = Math.floor(this.StopwatchPayload.elapsedMs / 1000);
            this.hours = Math.floor(totalSeconds / 3600);
            this.minutes = Math.floor(totalSeconds / 60) % 60;
            this.seconds = Math.floor(totalSeconds % 60);
            this.stopwatchDisplay =
                "Elapsed Time:\n" +
                String(this.hours).padStart(2, "0") +
                ":" +
                String(this.minutes).padStart(2, "0") +
                ":" +
                String(this.seconds).padStart(2, "0");
        });

        try {
            const result = await invoke<SessionRust>("start_tracker", { gameInput: this.gameInput, pid: this.pid });
            this.sessionData = result;
            this.headerMessage = "Session ended!";
        } catch (error) {
            this.errorMsg = String(error);
        }
    }

    pauseSession = async (event: Event) => {
        try {
            invoke("toggle_pause")
            this.paused = true;
        } catch (error) {
            this.errorMsg = String(error)
        }   
    }

    resumeSession = async (event: Event) => {
        try {
            invoke("toggle_resume")
            this.paused = false;
        } catch (error) {
            this.errorMsg = String(error)
        }   
    }

    endSession = async (event: Event) => {
        event.preventDefault();

        if (this.unlistenStopwatch) {
            this.unlistenStopwatch();
            this.unlistenStopwatch = null;
        }

        if (this.newGameInput.trim().length > 0) {
            this.sessionData.game = this.newGameInput;
        }

        try {
            await invoke("end_tracker", {
                sessionNotes: this.sessionNotes,
                sessionData: this.sessionData
            });

            this.successMsg = "Session data saved to database!";

            this.sessionData = {};
            this.gameInput = "";
            this.newGameInput = "";
            this.sessionNotes = "";
            this.headerMessage = "Enter a game title to get started";
            this.gameFound = false;
            this.searchResults = [];
            this.paused = false;

        } catch (error) {
            this.errorMsg = String(error);
        }
    }
}

export const tracker = new SessionTracker();