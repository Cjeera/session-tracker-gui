import { invoke } from "@tauri-apps/api/core";
import type { Session, GameTimeRangeStats } from "./types.svelte.js";

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

export async function getGameTimeRangeStats(gameId: number) {
    try {
        const [weeklyPlaytime, monthlyPlaytime, yearlyPlaytime] = await Promise.all([
            invoke<GameTimeRangeStats[]>("get_game_weekly_stats", { gameId: gameId }),
            invoke<GameTimeRangeStats[]>("get_game_monthly_stats", { gameId: gameId }),
            invoke<GameTimeRangeStats[]>("get_game_yearly_stats", { gameId: gameId }),
        ]);
        

    const weeklyPlaytimeStats = weeklyPlaytime.map((entry) => ({
        range: entry.range,
        playtime: Number((entry.totalPlaytime / 3600).toFixed(2))
    }));

    const monthlyPlaytimeStats = monthlyPlaytime.map((entry) => ({
        range: entry.range,
        playtime: Number((entry.totalPlaytime / 3600).toFixed(2))
    }));

    const yearlyPlaytimeStats = yearlyPlaytime.map((entry) => ({
        range: entry.range,
        playtime: Number((entry.totalPlaytime / 3600).toFixed(2))
    }));

    return { weeklyPlaytimeStats, monthlyPlaytimeStats, yearlyPlaytimeStats }

    } catch (err) {
        console.error(err);
    }
}

export function getPlaytimeDateString(firstTimestamp: string, lastTimestamp: string) {
    try {
        let firstDate = Temporal.Instant.from(firstTimestamp).toZonedDateTimeISO("UTC").toPlainDate();
        let secondDate = Temporal.Instant.from(lastTimestamp).toZonedDateTimeISO("UTC").toPlainDate();

        const diff = firstDate.until(secondDate, {
            largestUnit: "years"
        });

        const years = diff.years;
        const months = diff.months;
        const weeks = Math.trunc(diff.days / 7);
        const days = diff.days % 7; 

        if (years > 0) {
            return `${years} Years, ${months} Months, ${weeks} Weeks, ${days} Days`;
        } else if (months > 0) {
            return `${months} Months, ${weeks} Weeks, ${days} Days`;
        } else if (weeks > 0) {
            return `${weeks} Weeks, ${days} Days`;
        } else if (days > 0) {
            return `${days} Days`;
        } else {
            return "";
        }

    } catch (error) {
        console.error(error);
    }
}