
type Meta = {
    code: number,
    message: string
}
type FieldError = {
    name: string,
    error_message: string[]
}
type  ApiResponse<T> = {
    data: T,
    meta: Meta,
    errors: FieldError[] | null
}

type Env = Record<string, string>


async function apiFetch<T>(
    BASE_URL: string,
    token: string,
    endpoint: string,
    options: RequestInit = {}
): Promise<ApiResponse<T>> {
    const response = await fetch(`${BASE_URL}${endpoint}`, {
        ...options,
        headers: {
            'Content-Type': 'application/json',
            ...{'X-Bridge-Token': `Bearer ${token}`},
            ...options.headers
        }
    });

    try {
        if (!response.ok) {
            return {
                data: {} as T,
                errors: [],
                meta: {
                    code: response.status,
                    message: `${response.statusText}`
                }
            }
        }
        return response.json()
    } catch (e) {
        return {
            data: {} as T,
            errors: [],
            meta: {
                code: 400,
                message: `${e}`
            }
        }
    }
}

class NomiRpc {
    private BASE_URL = ""
    private token = ""
    private incoming: any
    private workspace: any
    private payload: any
    private env: Env= {};

    private constructor(
        BASE_URL: string,
        token: string,
        incoming: any,
        payload: any,
        workspace: any,
        env:Env
    ) {
        this.BASE_URL = BASE_URL
        this.token = token
        this.incoming = incoming
        this.workspace = workspace
        this.payload = payload
        this.env = env
    }

    static new(
        BASE_URL: string,
        token: string,
        incoming: any,
        payload: any,
        workspace: any,
        env:Env
    ) {
        return new NomiRpc(
            BASE_URL,
            token,
            incoming,
            payload,
            workspace,
            env
        )
    }

    async hybrid_retrieve_knowledge(
        query_text: string,
        start_date: string | null | undefined,
        end_date: string | null | undefined
    ) {
        const response = apiFetch(
            this.BASE_URL,
            this.token,
            "/api/internal/rpc/retrieve-knowledge",
            {
                method: "POST",
                body: JSON.stringify({
                    query: query_text,
                    start_date: start_date,
                    end_date: end_date,
                    conversation_id: this.workspace.id
                })
            }
        )
        return response
    }

    var(key:string):string|null{
       return  this.env[key]
    }
}

//@ts-ignore
const rpc: NomiRpc = NomiRpc.new(
    __BASE_URL__,
    __token__,
    __incoming__,
    __payload__,
    __workspace__,
    __env__
)

function get_env(key:string):string|null|undefined{
    return rpc.var(key)
}