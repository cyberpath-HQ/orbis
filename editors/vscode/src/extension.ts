/**
 * Orbis DSL VS Code Extension
 *
 * This extension provides language support for Orbis DSL files (.orbis) including:
 * - Syntax highlighting (via TextMate grammar)
 * - Semantic highlighting (via LSP)
 * - Code completion
 * - Hover documentation
 * - Go to definition
 * - Find references
 * - Diagnostics
 * - Document symbols (outline)
 * - Rename support
 * - Folding
 * - Code actions
 */

import * as path from "path";
import * as vscode from "vscode";
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

/**
 * Activate the extension
 */
export async function activate(
    context: vscode.ExtensionContext
): Promise<void> {
    outputChannel = vscode.window.createOutputChannel("Orbis LSP");

    outputChannel.appendLine("Orbis DSL extension activating...");

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand("orbis.restartServer", restartServer),
        vscode.commands.registerCommand("orbis.showOutput", () =>
            outputChannel.show()
        )
    );

    // Start the language server
    await startLanguageServer(context);

    outputChannel.appendLine("Orbis DSL extension activated");
}

/**
 * Deactivate the extension
 */
export async function deactivate(): Promise<void> {
    if (client) {
        await client.stop();
        client = undefined;
    }
}

/**
 * Start the language server
 */
async function startLanguageServer(
    context: vscode.ExtensionContext
): Promise<void> {
    const config = vscode.workspace.getConfiguration("orbis");

    // Find the LSP executable
    const lspPath = await findLspExecutable(config);

    if (!lspPath) {
        const message =
            "Orbis LSP executable not found. Please install it or configure orbis.lsp.path.";
        outputChannel.appendLine(`ERROR: ${message}`);
        vscode.window.showErrorMessage(message);
        return;
    }

    outputChannel.appendLine(`Using LSP executable: ${lspPath}`);

    // Server options
    const args: string[] = config.get<string[]>("lsp.args") || [];

    const serverOptions: ServerOptions = {
        command: lspPath,
        args,
        transport: TransportKind.stdio,
        options: {
            env: {
                ...process.env,
                RUST_LOG: getLogLevel(config),
            },
        },
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "orbis" }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher("**/*.orbis"),
        },
        outputChannel,
        traceOutputChannel: outputChannel,
        initializationOptions: {
            validation: {
                enabled: config.get<boolean>("validation.enabled", true),
            },
            completion: {
                enabled: config.get<boolean>("completion.enabled", true),
            },
            hover: {
                enabled: config.get<boolean>("hover.enabled", true),
            },
        },
    };

    // Create and start the client
    client = new LanguageClient(
        "orbis-lsp",
        "Orbis Language Server",
        serverOptions,
        clientOptions
    );

    outputChannel.appendLine("Starting Orbis Language Server...");

    try {
        await client.start();
        outputChannel.appendLine("Orbis Language Server started successfully");
    } catch (error) {
        const message = `Failed to start Orbis Language Server: ${error}`;
        outputChannel.appendLine(`ERROR: ${message}`);
        vscode.window.showErrorMessage(message);
    }
}

/**
 * Find the LSP executable path
 */
async function findLspExecutable(
    config: vscode.WorkspaceConfiguration
): Promise<string | undefined> {
    // Check configuration first
    const configuredPath = config.get<string>("lsp.path");
    if (configuredPath && configuredPath.length > 0) {
        // Verify the path exists
        try {
            await vscode.workspace.fs.stat(vscode.Uri.file(configuredPath));
            return configuredPath;
        } catch {
            outputChannel.appendLine(
                `Configured LSP path not found: ${configuredPath}`
            );
        }
    }

    // Common installation locations
    const candidates: string[] = [];

    // Cargo install location
    const cargoHome = process.env.CARGO_HOME || path.join(homedir(), ".cargo");
    candidates.push(
        path.join(cargoHome, "bin", process.platform === "win32" ? "orbis-lsp.exe" : "orbis-lsp")
    );

    // Local build (for development)
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            candidates.push(
                path.join(
                    folder.uri.fsPath,
                    "target",
                    "debug",
                    process.platform === "win32" ? "orbis-lsp.exe" : "orbis-lsp"
                )
            );
            candidates.push(
                path.join(
                    folder.uri.fsPath,
                    "target",
                    "release",
                    process.platform === "win32" ? "orbis-lsp.exe" : "orbis-lsp"
                )
            );
        }
    }

    // System PATH
    candidates.push(process.platform === "win32" ? "orbis-lsp.exe" : "orbis-lsp");

    // Try each candidate
    for (const candidate of candidates) {
        try {
            // For PATH-based lookup, we can't stat it directly
            if (!path.isAbsolute(candidate)) {
                // Try to resolve from PATH using 'which' equivalent
                const { execSync } = require("child_process");
                try {
                    const resolved = execSync(
                        process.platform === "win32"
                            ? `where ${candidate}`
                            : `which ${candidate}`,
                        { encoding: "utf8" }
                    )
                        .split("\n")[0]
                        .trim();
                    if (resolved) {
                        outputChannel.appendLine(`Found LSP in PATH: ${resolved}`);
                        return resolved;
                    }
                } catch {
                    // Not found in PATH
                }
            } else {
                await vscode.workspace.fs.stat(vscode.Uri.file(candidate));
                outputChannel.appendLine(`Found LSP at: ${candidate}`);
                return candidate;
            }
        } catch {
            // Continue to next candidate
        }
    }

    return undefined;
}

/**
 * Get home directory cross-platform
 */
function homedir(): string {
    return (
        process.env.HOME ||
        process.env.USERPROFILE ||
        (process.env.HOMEDRIVE && process.env.HOMEPATH
            ? process.env.HOMEDRIVE + process.env.HOMEPATH
            : "/")
    );
}

/**
 * Get log level from configuration
 */
function getLogLevel(config: vscode.WorkspaceConfiguration): string {
    const trace = config.get<string>("trace.server", "off");
    switch (trace) {
        case "verbose":
            return "orbis_lsp=trace";
        case "messages":
            return "orbis_lsp=debug";
        default:
            return "orbis_lsp=info";
    }
}

/**
 * Restart the language server
 */
async function restartServer(): Promise<void> {
    outputChannel.appendLine("Restarting Orbis Language Server...");

    if (client) {
        await client.stop();
        client = undefined;
    }

    const context = await getExtensionContext();
    if (context) {
        await startLanguageServer(context);
    }
}

/**
 * Get extension context (cached from activation)
 */
let extensionContext: vscode.ExtensionContext | undefined;

function getExtensionContext(): Promise<vscode.ExtensionContext | undefined> {
    return Promise.resolve(extensionContext);
}

// Store context during activation
const originalActivate = activate;
export async function activate2(
    context: vscode.ExtensionContext
): Promise<void> {
    extensionContext = context;
    await originalActivate(context);
}
