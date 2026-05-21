<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import loader from '@monaco-editor/loader';

    let {
        value = $bindable(''),
        language = 'typescript',
        readOnly = false,
        typeDefinition = ''
    } = $props<{
        value: string;
        language?: string;
        readOnly?: boolean;
        typeDefinition?: string;
    }>();

    let editorContainer: HTMLElement;
    let editor: any;
    let monacoInstance: any;
    let currentDisposableLib: any;

    onMount(async () => {
        monacoInstance = await loader.init();

        // Configure TypeScript compiler options for the Edge Runtime
        monacoInstance.languages.typescript.typescriptDefaults.setCompilerOptions({
            target: monacoInstance.languages.typescript.ScriptTarget.ESNext,
            allowNonTsExtensions: true,
            moduleResolution: monacoInstance.languages.typescript.ModuleResolutionKind.NodeJs,
            module: monacoInstance.languages.typescript.ModuleKind.CommonJS,
            noEmit: true,
            typeRoots: ["node_modules/@types"]
        });

        // Define a custom dark theme to match Nomi's aesthetic
        monacoInstance.editor.defineTheme('nomi-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [],
            colors: {
                'editor.background': '#0d1117',
                'editor.lineHighlightBackground': '#161b22',
            }
        });

        editor = monacoInstance.editor.create(editorContainer, {
            value,
            language,
            theme: 'nomi-dark',
            automaticLayout: true,
            readOnly,
            minimap: { enabled: false },
            fontSize: 13,
            fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
            scrollBeyondLastLine: false,
            roundedSelection: false,
            padding: { top: 16, bottom: 16 }
        });

        editor.onDidChangeModelContent(() => {
            const currentVal = editor.getValue();
            if (value !== currentVal) {
                value = currentVal;
            }
        });
        
        updateTypeDefinition();
    });

    function updateTypeDefinition() {
        if (!monacoInstance) return;

        if (currentDisposableLib) {
            currentDisposableLib.dispose();
        }

        const libSource = `
            ${typeDefinition}
            
            /**
             * The secure token used to authorize internal RPC calls back to the gateway.
             */
            declare const BRIDGE_TOKEN: string;
            
            /**
             * Accesses your Rust gateway's native vector data pipeline.
             */
            declare function callInternalKnowledgeBase(query: string, limit?: number): Promise<any>;
        `;
        const libUri = "ts:filename/nomi-edge.d.ts";
        currentDisposableLib = monacoInstance.languages.typescript.typescriptDefaults.addExtraLib(libSource, libUri);
    }

    $effect(() => {
        if (editor && value !== editor.getValue()) {
            editor.setValue(value);
        }
    });

    $effect(() => {
        if (typeDefinition !== undefined) {
            updateTypeDefinition();
        }
    });

    onDestroy(() => {

        if (editor) {
            editor.dispose();
        }
    });
</script>

<div bind:this={editorContainer} class="w-full h-full"></div>
