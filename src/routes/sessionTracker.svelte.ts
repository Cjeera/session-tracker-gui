import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { SessionRust, Process } from "$lib/types";

// A class containing all session tracking logic.
class SessionTracker {
    unlistenStopwatch: UnlistenFn | null = null;
    unlistenSessionEnd: UnlistenFn | null = null;

    // Seconds, minutes and hours, which will be formatted into HH:MM:SS.
    seconds = $state(0);
    minutes = $state(0);
    hours = $state(0);

    // The stopwatch that will display once tracking begins.
    stopwatchDisplay = $state("");

    // A process array that search results will be stored in.
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

    sessionData = $state<Partial<SessionRust>>({});
    pid = $state(0);

    StopwatchPayload = $state({
        elapsedMs: 0,
    });

    searchProcesses = async (event: Event) => {
        event.preventDefault();

        // Error flag, search results, stopwatch, success message and search successful are reset.
        this.errorFlag = false;      
        this.searchResults = [];
        this.stopwatchDisplay = "Elapsed Time: 00:00:00";
        this.successMsg = "";
        this.searchSuccessful = false;

        // Forbids game inputs equal to or less than 1. 
        if (this.gameInput.length <= 1) {
            this.errorFlag = true;
            this.errorMsg = "Enter a name at least 2 characters long";
            return;
        }

        // Calls the search_processes backend function, sending gameInput as an argument.
        try {
            this.searchResults = await invoke<Process[]>("search_processes", { gameInput: this.gameInput });
            this.searchSuccessful = true;
        } catch (error) {
            this.errorFlag = true;
            this.errorMsg = String(error);
            console.error(error);
        }
    }

    trackSession = async (process: Process) => {
        this.paused = false;
        this.searchSuccessful = false;
        this.errorFlag = false;
        this.gameFound = true;

        this.pid = process.pid      
        this.gameInput = process.name; 

        // Changes the header message.
        this.headerMessage = "Currently Tracking " + this.gameInput + "\n(PID " + this.pid +")";

        // Backend stopwatch is reset
        if (this.unlistenStopwatch) {
            this.unlistenStopwatch();
        }

        // Backend session is reset
        if (this.unlistenSessionEnd) {
            this.unlistenSessionEnd();
        }

        // A listener is set up which will get the elapsed time in ms.
        this.unlistenStopwatch = await listen<{ elapsedMs: number }>("stopwatch-tick", (event) => {
            // Payload is recieved from the backend.
            this.StopwatchPayload.elapsedMs = event.payload.elapsedMs;

            // Calculates total seconds.
            let totalSeconds = Math.floor(this.StopwatchPayload.elapsedMs / 1000);

            // Calculates hours.
            this.hours = Math.floor(totalSeconds / 3600);

            // Calculates minutes.
            this.minutes = Math.floor(totalSeconds / 60) % 60;

            // Calculates seconds.
            this.seconds = Math.floor(totalSeconds % 60);

            // Formats into HH:MM:SS.
            this.stopwatchDisplay =
                "Elapsed Time:\n" +
                String(this.hours).padStart(2, "0") +
                ":" +
                String(this.minutes).padStart(2, "0") +
                ":" +
                String(this.seconds).padStart(2, "0");
        });

        // A listener is set up for the end of a session.
        this.unlistenSessionEnd = await listen<SessionRust>("session-ended", (event) => {
            this.sessionData = event.payload;
            this.headerMessage = "Session ended!"
        })

        // Calls start_tracker backend function, sending gameInput, gameInput and pid as arguments.
        try {
            await invoke("start_tracker", { gameInput: this.gameInput, pid: this.pid });
        } catch (error) {
            this.errorMsg = String(error);
            this.gameFound = false;
        }
    }

    pauseSession = async () => {
        // Calls toggle_pause backend function.
        try {
            await invoke("toggle_pause")
            this.paused = true;
        } catch (error) {
            this.errorMsg = String(error)
        }   
    }

    resumeSession = async () => {
        // Calls toggle_resume backend function.
        try {
            await invoke("toggle_resume")
            this.paused = false;
        } catch (error) {
            this.errorMsg = String(error)
        }   
    }

    userStopSession = async () => {
        try {
            await invoke("toggle_end")
        } catch (error) {
            this.errorMsg = String(error)
        }
    }

    endSession = async (event: Event) => {
        event.preventDefault();

        // Stops the stopwatch.
        if (this.unlistenStopwatch) {
            this.unlistenStopwatch();
            this.unlistenStopwatch = null;
        }

        if (this.unlistenSessionEnd) {
            this.unlistenSessionEnd();
            this.unlistenSessionEnd = null;
        }

        // If the user entered a new game title, assigns it to sessionData.game.
        if (this.newGameInput.trim().length > 0) {
            this.sessionData.game = this.newGameInput;
        }

        try {
            // Calls end_tracker backend function, sending sessionNotes and sessionData as arguments.
            await invoke("end_tracker", {
                sessionNotes: this.sessionNotes,
                sessionData: this.sessionData
            });

            // Success message is displayed.
            this.successMsg = "Session data saved to database!";

            this.sessionData = {};
            this.gameInput = "";
            this.newGameInput = "";
            this.sessionNotes = "";
            this.headerMessage = "Enter a game title to get started";
            this.gameFound = false;
            this.searchResults = [];
            this.paused = false;
            this.stopwatchDisplay = "";

        } catch (error) {
            this.errorMsg = String(error);
        }
    }
}

// tracker is exported for use as an instance of SessionTracker.
export const tracker = new SessionTracker();