export function formatDate(timestampString: string) {
    const date = new Date(timestampString);

    return date.toDateString();
}

export function formatLocaleDate(timestampString: string) {
    const date = new Date(timestampString);

    return date.toLocaleDateString();
}

export function formatTime(timestampString: string) {
    const date = new Date(timestampString);

    return date.toLocaleTimeString();
}

export function formatDuration(durationSeconds: number) {
    let hours = Math.floor(durationSeconds / 3600);
    let minutes = Math.floor((durationSeconds % 3600) / 60);
    let seconds = durationSeconds % 60;
    
    return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}