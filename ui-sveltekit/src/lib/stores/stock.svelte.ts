import { eventBus } from '$lib/utils';

export interface StockSignal {
    ticker: string;
    price: number;
    rsi: number;
    macd_hist: number;
    bb_upper: number;
    bb_lower: number;
    signal: 'STRONG BUY' | 'STRONG SELL' | 'NEUTRAL';
    timestamp: number;
    lastUpdated?: number;
}

class StockStore {
    signals = $state<Record<string, StockSignal>>({});
    currentTime = $state(Date.now());

    constructor() {
        if (typeof window !== 'undefined') {
            this.init();
        }
    }

    private init() {
        eventBus.subscribe('stock-signal', (data: StockSignal) => {
            this.signals[data.ticker] = {
                ...data,
                lastUpdated: Date.now()
            };
        });

        setInterval(() => {
            this.currentTime = Date.now();
        }, 1000);
    }

    get sortedSignals() {
        return Object.values(this.signals).sort((a, b) => {
            return b.timestamp - a.timestamp;
        });
    }

    getSecondsAgo(timestamp: number | undefined) {
        if (!timestamp) return 'N/A';
        // The timestamp from backend is in seconds, converting to ms for comparison with Date.now()
        // Or if backend sends seconds, we should be consistent.
        // Assuming backend sends unix timestamp in seconds.
        const nowInSeconds = Math.floor(this.currentTime / 1000);
        const seconds = nowInSeconds - timestamp;
        return seconds < 0 ? 0 : seconds;
    }
}

export const stockStore = new StockStore();
