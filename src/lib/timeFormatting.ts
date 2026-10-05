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
    const totalSeconds = Math.round(durationSeconds);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    return [hours, minutes, seconds]
        .map(value => String(value).padStart(2, "0"))
        .join(":");
}