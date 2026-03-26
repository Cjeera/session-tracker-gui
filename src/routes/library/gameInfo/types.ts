export interface Session {
        sessionId: number;
        startTs: string;
        endTs: string;
        durationSeconds: number
        notes: string;
    }

export interface Game {
        gameId: number;
        title: string;
        coverPath: string;
    }

export interface GameStats {
        gameId: number;
        totalPlaytime: number;
        totalSessions: number;
        lastPlayed: string;
    }