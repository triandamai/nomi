/**
 * Generates the TypeScript type definitions for the Nomi Edge runtime.
 * This includes the BunArgs interface (based on the JSON schema),
 * InboundMessage, Workspace, and the global NomiArgs object.
 */
export function generateNomiTypeDefinition(schemaJson: string): string {
    try {
        const schema = JSON.parse(schemaJson);
        const props = schema.properties || {};

        let tsInterface = "/** Auto-generated payload from your JSON Schema. */\ninterface NomiPayload {\n";
        for (const [key, config] of Object.entries(props)) {
            const conf = config as any;
            const isOptional = schema.required && !schema.required.includes(key);
            let tsType = 'any';
            if (conf.type === 'string') tsType = 'string';
            else if (conf.type === 'integer' || conf.type === 'number') tsType = 'number';
            else if (conf.type === 'boolean') tsType = 'boolean';
            else if (conf.type === 'array') tsType = 'any[]';
            else if (conf.type === 'object') tsType = 'Record<string, any>';

            if (conf.description) tsInterface += `    /** ${conf.description} */\n`;
            tsInterface += `    ${key}${isOptional ? '?' : ''}: ${tsType};\n`;
        }
        tsInterface += "}\n\n";

        tsInterface += "";

        tsInterface += `
    interface InboundMessage {
        is_group: boolean;
        is_private: boolean;
        is_mentioned: boolean;
        sender_id: string;
        conversation_id: string;
        message_id: string;
        text: string;
        channel: string;
        image_url?: string;
        video_url?: string;
        audio_url?: string;
    }
    interface Workspace {id: string;title: string;}
    interface NomiArgs {incoming: InboundMessage;payload: NomiPayload;workspace: Workspace;}
    type Meta = {
        code: number;
        message: string;
    };

    type FieldError = { 
        name: string; 
        error_message: string[]; 
    };

    type ApiResponse<T> = {
        data: T;
        meta: Meta;
        errors: FieldError[] | null;
    };

    type Env = Record<string, string>;

    /**
     * Internal utility fetch router with automatic header synchronization handles
     */
    declare function apiFetch<T>(
        BASE_URL: string,
        token: string,
        endpoint: string,
        options?: RequestInit
    ): Promise<ApiResponse<T>>;

    /**
     * Core Nomi RPC Interface Engine.
     * Provides first-class abstractions to query memories, environmental vars, and system states.
     */
    class NomiRpc {
        private BASE_URL: string;
        private token: string;
        private incoming: any;
        private workspace: any;
        private payload: any;
        private env: Env;

        private constructor(
            BASE_URL: string,
            token: string,
            incoming: any,
            payload: any,
            workspace: any,
            env: Env
        );

        /**
         * Instantiates a specialized instance of the NomiRpc runtime execution controller.
         */
        static new(
            BASE_URL: string,
            token: string,
            incoming: any,
            payload: any,
            workspace: any,
            env: Env
        ): NomiRpc;

        /**
         * Queries global or room-specific knowledge base RAG matrices across conversation boundaries.
         * @param query_text The search query phrasing or question semantic token.
         * @param start_date Optional timeline slice lower bound filter string.
         * @param end_date Optional timeline slice upper bound filter string.
         */
        hybrid_retrieve_knowledge(
            query_text: string,
            start_date?: string | null,
            end_date?: string | null
        ): Promise<ApiResponse<any>>;

        /**
         * Safely extracts an environmental variable variable value mapping straight from the container.
         * @param key The absolute string identifier name of your configured plugin variable target.
         */
        var(key: string): string | null;
    }
    
   
    /**
     * Pre-hydrated, active instance of the NomiRpc subsystem mapped explicitly to the execution pipeline turn.
     */
    declare const rpc: NomiRpc;
      /**
     * Pre-hydrated, active instance of the NomiRpc subsystem mapped explicitly to the execution pipeline turn.
     */
    
     declare function get_env(key:string):string|null|undefined{
         return rpc.var(key)
     }
  
`;

        return tsInterface;
    } catch {
        return "interface NomiArgs { incoming: any; payload: any; workspace: any; }\ndeclare function retrieve_knowledge(query: string, limit?: number): Promise<any>;";
    }
}
