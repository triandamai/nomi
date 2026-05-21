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

        tsInterface += `interface InboundMessage {
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
}\n\n`;

        tsInterface += `interface Workspace {
    id: string;
    title: string;
}\n\n`;

        tsInterface += `interface NomiArgs {
    incoming: InboundMessage;
    payload: NomiPayload;
    workspace: Workspace;
}\n\n`;

        tsInterface += "/** Built-in: Semantic Knowledge Retrieval */\ndeclare function retrieve_knowledge(query: string, limit?: number): Promise<any>;";
        
        return tsInterface;
    } catch {
        return "interface NomiArgs { incoming: any; payload: any; workspace: any; }\ndeclare function retrieve_knowledge(query: string, limit?: number): Promise<any>;";
    }
}
