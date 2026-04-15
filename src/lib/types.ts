export interface Session {
        sessionId: number;
        startTs: string;
        endTs: string;
        durationSeconds: number
        notes: string;
    }

export interface SessionRust {
        game: string;
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

export interface Process {
        pid: number;
        name: string;
    }

export interface GameCover {
    cover?: Cover;
}

export interface Cover {
    url: string;
}
export const config = {
    coverArtChoice: true
};
