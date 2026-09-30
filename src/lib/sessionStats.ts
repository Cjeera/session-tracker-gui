import type { Session } from "./types.js";

/** Aggregate sessions by local calendar day, returning the seven most recent active days. */
export function getRecentActivity(sessions: Session[]) {
    const days = new Map<string, { day: string; date: string; timestamp: number; sessions: number }>();
    for (const session of sessions) {
        const start = new Date(session.startTs);
        const timestamp = new Date(start.getFullYear(), start.getMonth(), start.getDate()).getTime();
        const day = String(timestamp);
        const entry = days.get(day);
        if (entry) entry.sessions += 1;
        else days.set(day, { day, date: start.toLocaleDateString(), timestamp, sessions: 1 });
    }
    return [...days.values()].sort((a, b) => a.timestamp - b.timestamp).slice(-7);
}

/** Use a unique session key so sessions on the same date remain separate bars. */
export function getLongestSessions(sessions: Session[]) {
    return [...sessions]
        .sort((a, b) => b.durationSeconds - a.durationSeconds)
        .slice(0, 5)
        .map((session) => ({
            key: String(session.sessionId),
            date: new Date(session.startTs).toLocaleDateString(),
            durationSeconds: session.durationSeconds,
        }));
}

/** Keep full second precision so brief sessions still contribute to the pie chart. */
export function getWeekdayPlaytime(sessions: Session[]) {
    const days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    const data = days.map((label) => ({ key: label.toLowerCase(), label, seconds: 0 }));
    for (const session of sessions) {
        data[new Date(session.startTs).getDay()].seconds += session.durationSeconds;
    }
    return data;
}
