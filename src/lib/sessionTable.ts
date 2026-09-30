import type { Session } from "./types.js";

export type SessionColumn = "displayId" | "startDate" | "endDate" | "startTime" | "endTime" | "durationSeconds";
export type SessionRow = Session & { displayId: number };
export type SortDirection = "ascending" | "descending";

function secondsOfDay(timestamp: string) {
    const date = new Date(timestamp);
    return date.getHours() * 3600 + date.getMinutes() * 60 + date.getSeconds();
}

function sortValue(session: SessionRow, column: SessionColumn) {
    switch (column) {
        case "startDate": return new Date(session.startTs).getTime();
        case "endDate": return new Date(session.endTs).getTime();
        case "startTime": return secondsOfDay(session.startTs);
        case "endTime": return secondsOfDay(session.endTs);
        default: return session[column];
    }
}

/** Number rows before sorting, retaining the backend session ID for selection and editing. */
export function getSessionRows(sessions: Session[], column: SessionColumn | null, direction: SortDirection) {
    const rows: SessionRow[] = sessions.map((session, index) => ({ ...session, displayId: index + 1 }));
    if (!column) return rows;
    const multiplier = direction === "ascending" ? 1 : -1;
    return rows.sort((a, b) => multiplier * (sortValue(a, column) - sortValue(b, column)) || a.displayId - b.displayId);
}

/** Clamp the page when new data contains fewer sessions. */
export function getSessionPage(rows: SessionRow[], pageIndex: number, pageSize: number) {
    const pageCount = Math.max(1, Math.ceil(rows.length / pageSize));
    const currentPage = Math.max(0, Math.min(pageIndex, pageCount - 1));
    return {
        currentPage,
        pageCount,
        rows: rows.slice(currentPage * pageSize, (currentPage + 1) * pageSize),
    };
}
