import {
    useState, useEffect
} from "react";
import { PluginRenderer } from "./components";
import {
    healthCheck, getMode, getPlugins, getPluginPages
} from "./api";
import type {
    AppModeInfo, PluginInfo, PluginPage
} from "./types";
import "./App.css";
import "./styles/plugin.css";

function App() {
    const [
        status,
        setStatus,
    ] = useState<`loading` | `connected` | `error`>(`loading`);
    const [
        mode,
        setMode,
    ] = useState<AppModeInfo | null>(null);
    const [
        plugins,
        setPlugins,
    ] = useState<Array<PluginInfo>>([]);
    const [
        pluginPages,
        setPluginPages,
    ] = useState<Array<PluginPage>>([]);
    const [
        activePage,
        setActivePage,
    ] = useState<PluginPage | null>(null);
    const [
        error,
        setError,
    ] = useState<string | null>(null);

    useEffect(() => {
        async function init() {
            try {
                // Check health
                await healthCheck();
                setStatus(`connected`);

                // Get mode info
                const modeInfo = await getMode();
                setMode(modeInfo);

                // Get plugins
                const {
                    plugins: loadedPlugins,
                } = await getPlugins();
                setPlugins(loadedPlugins);

                // Get plugin pages
                const {
                    pages,
                } = await getPluginPages();
                setPluginPages(pages);
            }
            catch (err) {
                setStatus(`error`);
                setError(err instanceof Error ? err.message : String(err));
            }
        }

        init();
    }, []);

    if (status === `loading`) {
        return (
            <main className="container">
                <div className="loading">
                    <h2>Loading Orbis...</h2>
                    <div className="spinner" />
                </div>
            </main>
        );
    }

    if (status === `error`) {
        return (
            <main className="container">
                <div className="error">
                    <h2>Failed to connect</h2>
                    <p>{error}</p>
                    <button onClick={() => window.location.reload()}>Retry</button>
                </div>
            </main>
        );
    }

    // If a plugin page is active, render it
    if (activePage) {
        return (
            <main className="container">
                <header className="header">
                    <button className="back-button" onClick={() => setActivePage(null)}>
                        ← Back
                    </button>
                    <h1>{activePage.title}</h1>
                </header>
                <div className="plugin-content">
                    <PluginRenderer
                        schema={activePage.layout}
                        data={{}}
                        handlers={{}}
                    />
                </div>
            </main>
        );
    }

    // Main dashboard
    return (
        <main className="container">
            <header className="header">
                <h1>Orbis</h1>
                <span className="mode-badge">
                    {mode?.mode || `unknown`}
                </span>
            </header>

            <section className="section">
                <h2>Status</h2>
                <div className="status-grid">
                    <div className="status-card">
                        <span className="status-label">Mode</span>
                        <span className="status-value">{mode?.mode}</span>
                    </div>
                    <div className="status-card">
                        <span className="status-label">Plugins</span>
                        <span className="status-value">{plugins.length}</span>
                    </div>
                </div>
            </section>

            {plugins.length > 0 && (
                <section className="section">
                    <h2>Plugins</h2>
                    <div className="plugin-list">
                        {plugins.map((plugin) => (
                            <div key={plugin.id} className="plugin-card">
                                <div className="plugin-info">
                                    <h3>{plugin.name}</h3>
                                    <p>{plugin.description}</p>
                                    <span className={`plugin-state plugin-state--${ plugin.state.toLowerCase() }`}>
                                        {plugin.state}
                                    </span>
                                </div>
                                <span className="plugin-version">v{plugin.version}</span>
                            </div>
                        ))}
                    </div>
                </section>
            )}

            {pluginPages.filter((p) => p.show_in_menu).length > 0 && (
                <section className="section">
                    <h2>Plugin Pages</h2>
                    <nav className="page-nav">
                        {pluginPages
                            .filter((p) => p.show_in_menu)
                            .sort((a, b) => (a.menu_order ?? 0) - (b.menu_order ?? 0))
                            .map((page) => (
                                <button
                                    key={page.route}
                                    className="page-link"
                                    onClick={() => setActivePage(page)}
                                >
                                    {page.icon && <span className="page-icon">{page.icon}</span>}
                                    <span className="page-title">{page.title}</span>
                                    {page.description && (
                                        <span className="page-description">{page.description}</span>
                                    )}
                                </button>
                            ))}
                    </nav>
                </section>
            )}

            {plugins.length === 0 && (
                <section className="section empty-state">
                    <h2>No Plugins Loaded</h2>
                    <p>Install plugins to extend Orbis functionality.</p>
                </section>
            )}
        </main>
    );
}

export default App;
