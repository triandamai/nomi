import { chatApi } from '$lib/api/client';

export type HealthMetric = {
    id: string;
    user_id: string;
    log_date: string;
    metrics: {
        steps?: number;
        sleep_hours?: number;
        avg_heart_rate?: number;
        active_minutes?: number;
        workout?: string;
        note?: string;
        [key: string]: any;
    };
    created_at: string;
    updated_at: string;
};

function createHealthStore() {
    let history = $state<HealthMetric[]>([]);
    let loading = $state(false);
    let startDate = $state(new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]);
    let endDate = $state(new Date().toISOString().split('T')[0]);

    async function fetchHistory() {
        loading = true;
        try {
            const res = await chatApi.getHealthHistory(startDate, endDate);
            history = res.data;
        } catch (e) {
            console.error('Failed to fetch health history', e);
        } finally {
            loading = false;
        }
    }

    return {
        get history() {
            return history;
        },
        get loading() {
            return loading;
        },
        get startDate() {
            return startDate;
        },
        set startDate(value: string) {
            startDate = value;
        },
        get endDate() {
            return endDate;
        },
        set endDate(value: string) {
            endDate = value;
        },
        get stats() {
            if (history.length === 0) return { steps: 0, sleep: 0, heart: 0 };
            const latest = history[history.length - 1].metrics;
            return {
                steps: latest.steps || 0,
                sleep: latest.sleep_hours || 0,
                heart: latest.avg_heart_rate || 0
            };
        },
        get stepsData() {
            return history.map(h => h.metrics.steps || 0);
        },
        fetchHistory
    };
}

export const healthStore = createHealthStore();
