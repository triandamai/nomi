import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params, fetch }) => {
    const { slug } = params;
    
    // Fetch the current reinforcement state for the given plugin slug
    const res = await fetch(`/api/srp/${slug}`);
    
    if (!res.ok) {
        if (res.status === 404) {
            // Return empty state if no reinforcement exists yet
            return {
                slug,
                enriched_description: "Original static definition active. No autonomous optimizations detected.",
                additional_rules: [],
                learned_phrases: []
            };
        }
        throw error(res.status, "Failed to load SRP data");
    }

    const reinforced = await res.json();
    return {
        slug,
        enriched_description: reinforced.enriched_description,
        additional_rules: reinforced.additional_rules,
        learned_phrases: reinforced.learned_phrases
    };
};
