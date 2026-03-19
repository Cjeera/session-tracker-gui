import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

class SessionTracker {

    // A variable to store the unlisten function from the stopwatch-tick listener.
    unlistenStopwatch;

    // Seconds, minutes and hours, which will be formatted into HH:MM:SS.
    seconds = $state(0);
    minutes = $state(0);
    hours = $state(0);

    // The stopwatch variable that will be displayed in the frontend.
    stopwatchDisplay = $state("Elapsed Time: 00:00:00");

    // A variable that will contain the game title input to be sent to the backend.
    gameInput = $state("");

    // A bool that displays an error in the game-input field if set to true
    errorFlag = $state(false);

    // The header text that displays at the top of the page.
    headerMessage = $state("Enter a game title to get started");

    // A variable that will contain the new title input to be sent to the backend should user choose to overwrite original input.
    newGameInput = $state("");

    // A variable that will contain user inputted session notes.
    sessionNotes = $state("");

    // A variable for diaplaying error messages.
    errorMsg = $state("");

    // A variable for displaying success messages.
    successMsg = $state("");

    // a bool that displays either the game input form (false) or the stopwatch (true).
    gameFound = $state(false);

    // An object that will store the session data struct returned from the backend.
    sessionData = $state({});

    // The found process ID.
    pid = $state(0);

    // An object for the elapsed time sent from the backend.
    StopwatchPayload = $state({
        elapsedMs: 0,
    });

    trackSession = async (event) => {
        event.preventDefault();
        
        // Stopwatch is reset.
        this.stopwatchDisplay = "Elapsed Time: 00:00:00";

        // Success message is reset.
        this.successMsg = "";

        // Error flag is reset, and result message is cleared.
        this.errorFlag = false;

        // Any input less or equal to characters will be rejected.
        if (this.gameInput.length <= 2) {
            this.errorFlag = true;
            this.resultMsg = "Enter a name at least 3 characters long";
            return;
        }

        // gameInput is sent to the backend.
        this.pid = await invoke("search_processes", { gameInput: this.gameInput }).catch((error) => {
            this.errorFlag = true;
            this.resultMsg = error;
            console.error(error);
            return;
        });

        // Function returns if no process ID is found.
        if (!this.pid) {
            this.errorFlag = true;
            this.resultMsg = "Game not found";
            return;
        }

        // Error flag is reset, and game found is set to true.
        this.errorFlag = false;
        this.gameFound = true;

        // Header message displays confirmation message.
        this.headerMessage =
            "Found " + this.gameInput + " (PID " + this.pid + ")! Tracking Started";

        // The elapsed time in milliseconds is taken from backend and formatted into HH:MM:SS
        this.unlistenStopwatch = await listen("stopwatch-tick", (event) => {
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

        // The main tracker backend logic is called.
        const result = await invoke("start_tracker", { gameInput: this.gameInput, pid: this.pid }).catch(
            (error) => {
                this.resultMsg = error;
                console.error(error);
            },
        );

        // Only assign data if the invoke was successful
        if (result) {
            this.sessionData = result;
            this.headerMessage = "Session ended!";
        }
    }

    // The sessionData object and the sessionNotes string are sent to the backend for database insertion.
    endSession = async (event) => {
        event.preventDefault();

        if (this.unlistenStopwatch) {
            this.unlistenStopwatch();
            this.unlistenStopwatch = null;
        }

        if (this.newGameInput.trim().length > 0) {
            this.sessionData.game = this.newGameInput;
        }

        try {
            // Try to save to the database
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

        } catch (error) {
            // If database save fails, the code jumps here.
            this.resultMsg = error;
            console.error("Database save failed:", error);
        }
    }
}
export const tracker = new SessionTracker();
